//! VW Transport protocol 2
//! 
//! This piggy-backs off an open CAN Channel, to provide a PayloadChannel
//! that can be used by the Diagnostic server

use std::{marker::PhantomData, sync::{Arc, Mutex, atomic::AtomicBool, mpsc}, thread::JoinHandle, time::{Duration, Instant}};

use crate::channel::{CanChannel, CanFrame, ChannelError, ChannelResult, Packet, PacketChannel, PayloadChannel, VwTp2Settings};

enum ChannelState {
    Uninit,
    Ready(Box<dyn CanChannel>),
    Running {
        handle: JoinHandle<Box<dyn CanChannel>>,
        is_running: Arc<AtomicBool>,
        write_tx: mpsc::Sender<Vec<u8>>,
        write_tx_resp: Mutex<mpsc::Receiver<ChannelResult<()>>>,
        read_rx: Mutex<mpsc::Receiver<Vec<u8>>>,
    }
}

/// Since VWTP2 can have different 'application' setups,
/// this trait is used for defining the application encoding
/// and decoding of data over the channel. It also defines
/// the Protocol ID byte of the broadcast messages
pub trait VwApplicationProtocol: Send + Sync {
    /// Defined protocol ID
    const PROTOCOL_ID: u8;
    /// Encode raw data to data to be sent over the TP channel
    fn encode_data(data: &[u8]) -> Vec<u8>;
    /// Decode TP channel bytes back to raw data
    fn decode_data(data: &[u8]) -> ChannelResult<Vec<u8>>;
}

/// VW Transport protocol for diagnostic application
#[derive(Debug, Clone, Copy)]
pub struct VwDiagnosticApplication;

impl VwApplicationProtocol for VwDiagnosticApplication {
    const PROTOCOL_ID: u8 = 0x01;

    fn encode_data(data: &[u8]) -> Vec<u8> {
        let len = data.len() as u16;
        let mut ret = len.to_be_bytes().to_vec();
        ret.extend_from_slice(data);
        ret
    }

    fn decode_data(data: &[u8]) -> ChannelResult<Vec<u8>> {
        if data.len() < 2 {
            Err(ChannelError::BufferEmpty)
        } else {
            let desired_len = u16::from_be_bytes(data[..2].try_into().unwrap()) as usize;
            if data.len() - 2 != desired_len {
                Err(ChannelError::Other(format!("Expected {} bytes, actually received {} bytes", desired_len, data.len()-2)))
            } else {
                Ok(data[2..].to_vec())
            }
        }
    }
}

/// VW Transport 2 channel
pub struct VwTransport2Channel<T: VwApplicationProtocol> {
    ecu_id: u8,
    settings: VwTp2Settings,
    tx_rx_ids: Option<(u16, u16)>,
    can: ChannelState,
    _phantom: PhantomData<T>
}

impl<T: VwApplicationProtocol> std::fmt::Debug for VwTransport2Channel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VwTransport2Channel")
            .field("ecu_id", &self.ecu_id)
            .field("settings", &self.settings)
            .field("tx_rx_ids", &self.tx_rx_ids)
            .finish()
    }
}

fn make_timing_byte(time: Duration) -> u8 {
    // Max value is 64
    let (unit, value) = if time.as_micros() < 640 {
        let val = (time.as_micros() / 10) as u8;
        (0, val)
    } else if time.as_millis() < 64 {
        // Clamp to 1ms min
        let clamped = time.as_millis().max(1).min(63);
        let val = (clamped) as u8;
        (1, val)
    } else if time.as_millis() < 640 {
        // Clamp to 10ms min
        let clamped = time.as_millis().max(10).min(630);
        let val = (clamped / 10) as u8;
        (2, val)
    } else {
        let clamped = time.as_millis().min(6300);
        let val = (clamped / 100) as u8;
        (3, val)
    };
    unit << 6 | value
}

fn decode_timing_byte(v: u8) -> Duration {
    let value = v & 0b111111;
    match (v >> 6) & 0b11 {
        0 => Duration::from_micros(100*(value as u64)),
        1 => Duration::from_millis(value as u64),
        2 => Duration::from_millis(10*(value as u64)),
        3 => Duration::from_millis(100*(value as u64)),
        _ => unreachable!()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ThreadState {
    Idle,
    TxInProgress {
        data: Vec<u8>,
        pos: usize,
        tx_count: usize,
    },
    WaitForTxAck {
        ack_timer: Instant,
        expected_packet_id: u8,
        continued_data: Option<(Vec<u8>, usize)>
    },
    MultiRx {
        expected_pci: u8,
        buf: Vec<u8>,
    }
}

impl<T: VwApplicationProtocol> VwTransport2Channel<T> {
    /// Create a new VW Transport channel.
    /// 
    /// ## Parameters
    /// * can - Underlying CAN channel to use
    /// * ecu_id - Target ECU ID, usually 0x00-0xEF
    /// * settings - VW Transport settings
    pub fn new(can: Box<dyn CanChannel>, ecu_id: u8, settings: VwTp2Settings) -> Self {
        Self {
            ecu_id,
            settings,
            tx_rx_ids: None,
            can: ChannelState::Ready(can),
            _phantom: PhantomData::default()
        }
    }

    /// Releases the VW Transport channel,
    /// returning the underlying CAN channel
    pub fn release(self) -> Box<dyn CanChannel> {
        match self.can {
            ChannelState::Ready(can_channel) => can_channel,
            ChannelState::Running { handle, is_running, .. } => {
                is_running.store(false, std::sync::atomic::Ordering::Relaxed);
                handle.join().unwrap()
            },
            ChannelState::Uninit => unreachable!()
        }
    }

    fn start_background_thread(&mut self, mut bs: u8, mut ack_timeout: Duration, mut inter_packet_ms: Duration, settings: VwTp2Settings) {
        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_t = is_running.clone();
        let (tx_id, rx_id) = self.tx_rx_ids.unwrap();

        let (tx_write, rx_write) = mpsc::channel::<Vec<u8>>();
        let (tx_read, rx_read) = mpsc::channel::<Vec<u8>>();
        let (tx_write_resp, rx_write_resp) = mpsc::channel();

        let mut can = match std::mem::replace(&mut self.can, ChannelState::Uninit) {
            ChannelState::Ready(can) => can,
            _ => unreachable!()
        };

        fn next_packet_id(current: u8) -> u8 {
            let mut next = current + 1;
            if next > 0x0F {
                next = 0x00;
            }
            next
        }

        fn send_ack(can: &mut Box<dyn CanChannel>, tx_canid: u16, last_packet_id: u8) {
            let response_id = next_packet_id(last_packet_id);
            let cf = CanFrame::new(tx_canid as u32, &[0xB0 | response_id], false);
            let _ = can.write_packets(vec![cf], 0);
        }

        let joinhandle = std::thread::spawn(move || {
            log::debug!("Keep alive thread started");
            let mut state = ThreadState::Idle;
            let keep_alive = CanFrame::new(tx_id as u32, &[0xA3], false);
            let mut last_ping = Instant::now();
            let mut tx_packet_id = 0u8;
            let mut last_tx_time = Instant::now();

            // Tx data stuff

            while is_running_t.load(std::sync::atomic::Ordering::Relaxed) {
                if state == ThreadState::Idle && let Ok(to_write) = rx_write.try_recv() {
                    log::debug!("To send: {to_write:02X?}");
                    state = ThreadState::TxInProgress {
                        data: to_write, 
                        pos: 0, 
                        tx_count: 0,
                    };
                }

                if let ThreadState::TxInProgress { data, pos, tx_count } = &mut state
                && (last_tx_time.elapsed() >= inter_packet_ms) {
                    last_ping = Instant::now(); // Prevent pinging

                    let left = &data[*pos..];
                    let mut final_packet = false;
                    let mut frame_data = if left.len() <= 7 {
                        final_packet = true;
                        let mut v = vec![0x10 | tx_packet_id];
                        v.extend_from_slice(&left);
                        v
                    } else  {
                        let mut v = vec![0x20 | tx_packet_id];
                        // Always full packet
                        v.extend_from_slice(&left[..7]);
                        v
                    };
                    if !final_packet {
                        *pos += 7;
                        *tx_count += 1;
                        last_tx_time = Instant::now();

                        if *tx_count >= bs as usize {
                            // Tell ECU waiting for ACK
                            frame_data[0] = 0x00 | tx_packet_id;
                            state = ThreadState::WaitForTxAck { 
                                ack_timer: Instant::now(), 
                                expected_packet_id: next_packet_id(tx_packet_id), 
                                continued_data: Some((data.clone(), *pos)) 
                            }
                        }
                    } else {
                        state = ThreadState::WaitForTxAck { 
                            ack_timer: Instant::now(), 
                            expected_packet_id: next_packet_id(tx_packet_id), 
                            continued_data: None 
                        }
                    }
                    tx_packet_id = next_packet_id(tx_packet_id);
                    let frame = CanFrame::new(tx_id as u32, &frame_data, false);
                    if let Err(e) = can.write_packets(vec![frame], 0) {
                        state = ThreadState::Idle;
                        let _ = tx_write_resp.send(Err(e));
                    } else {
                        last_tx_time = Instant::now();
                    }
                }
                
                let recorded_packets = {
                    if last_ping.elapsed().as_millis() > 1000 {
                        log::debug!("Channel ping");
                        let _ = can.write_packets(vec![keep_alive.clone()], 0);
                        last_ping = Instant::now();
                        last_tx_time = Instant::now();
                    }
                    can.read_packets(1000, 0).unwrap_or_default()
                };

                let mut activity = false;
                for packet in recorded_packets.iter().filter(|x| x.get_address() == rx_id as u32) {
                    activity = true;
                    last_ping = Instant::now();
                    let data = packet.get_data();
                    let packet_id = data[0] & 0x0F;
                    log::debug!("Incomming data from ECU: {data:02X?}");
                    match data[0] & 0xF0 {
                        0x00 => {
                            // Awaiting ack
                            if let ThreadState::MultiRx { .. } = &mut state {
                                let frame = CanFrame::new(tx_id as u32, &[0xB0 | next_packet_id(packet_id)], false);
                                let _ = can.write_packets(vec![frame], 0);
                                last_tx_time = Instant::now();
                            } else {
                                log::error!("Received request for ack but aren't receiving anything?")
                            }
                        },
                        0x10 => {
                            if state == ThreadState::Idle {
                                // 1 packet Rx
                                let _ = tx_read.send(data[1..].to_vec());
                            } else if let ThreadState::MultiRx { expected_pci, buf } = &mut state {
                                // Last packet of a multi-packet Rx
                                if packet_id == *expected_pci {
                                    buf.extend_from_slice(&data[1..]);
                                    let _ = tx_read.send(buf.clone());
                                    state = ThreadState::Idle;
                                } else {
                                    log::error!("Expected packet ID {}, got {packet_id}", *expected_pci);
                                    state = ThreadState::Idle;
                                }
                            }
                            // Write ACK
                            send_ack(&mut can, tx_id, packet_id);
                        },
                        0x20 => {
                            // Multi Rx mode!
                            if state == ThreadState::Idle {
                                // New Rx response!
                                state = ThreadState::MultiRx { 
                                    expected_pci: packet_id + 1, 
                                    buf: data[1..].to_vec()
                                }
                            } else if let ThreadState::MultiRx { expected_pci, buf, .. } = &mut state {
                                if packet_id == *expected_pci {
                                    buf.extend_from_slice(&data[1..]);
                                }

                                *expected_pci += 1;
                                if *expected_pci > 0x0F {
                                    *expected_pci = 0;
                                }
                            }
                        },
                        0x30 => {},
                        0xB0 => {
                            if let ThreadState::WaitForTxAck { continued_data, .. } = &mut state {
                                if let Some((data, resume_pos)) = continued_data {
                                    state = ThreadState::TxInProgress { 
                                        data: data.clone(), 
                                        pos: *resume_pos, 
                                        tx_count: 0, 
                                    };
                                    last_tx_time = Instant::now() 
                                } else {
                                    let _ = tx_write_resp.send(Ok(()));
                                    state = ThreadState::Idle;
                                }
                            }
                        },
                        0x90 => {},
                        0xA0 => {
                            // Channel negotiation
                            if packet_id == 1 {
                                // Reload configuration
                                bs = data[1];
                                ack_timeout = decode_timing_byte(data[2]);
                                inter_packet_ms = decode_timing_byte(data[4]);
                                log::debug!("Timing settings updated: BS: {bs}, ACK Timeout: {ack_timeout:?}, ST_MIN: {inter_packet_ms:?}")
                            } else if packet_id == 3 {
                                // Channel test (We reply with response)
                                let config_req = &[
                                    0xA1,
                                    settings.bs,
                                    make_timing_byte(settings.ack_timeout),
                                    0xFF,
                                    make_timing_byte(settings.inter_packet_spacing_ms),
                                    0xFF
                                ];
                                let frame = CanFrame::new((tx_id) as u32, config_req, false);
                                let _ = can.write_packets(vec![frame], 0);
                                last_tx_time = Instant::now();
                            }
                        }
                        _ => log::error!("Unknown VW PCI: {data:02X?}")
                    }
                }

                // Handle timeouts
                if let ThreadState::WaitForTxAck { ack_timer, .. } = &mut state {
                    if ack_timer.elapsed() > ack_timeout {
                        let _ = tx_write_resp.send(Err(ChannelError::WriteTimeout));
                        state = ThreadState::Idle;
                    }
                }

                if tx_packet_id >= 0x10 {
                    tx_packet_id = 0x00;
                }
                if !activity {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            log::debug!("Keep alive thread killed");
            is_running_t.store(false, std::sync::atomic::Ordering::Relaxed);
            can
        });
        self.can = ChannelState::Running {
            handle: joinhandle,
            is_running,
            write_tx: tx_write,
            write_tx_resp: Mutex::new(rx_write_resp),
            read_rx: Mutex::new(rx_read)
        };
    }
}

impl<T: VwApplicationProtocol> PayloadChannel for VwTransport2Channel<T> {
    fn open(&mut self) -> ChannelResult<()> {
        if let Some((tx, rx)) = self.tx_rx_ids.as_mut() && let ChannelState::Ready(can) = &mut self.can {
            can.close()?;
            can.set_can_cfg(self.settings.can_baud, false)?; // Always false for VWTP
            can.open()?;
            can.clear_rx_buffer()?;
            can.clear_tx_buffer()?;
            // Step 1 - Broadcast - See if the ECU replies to us
            let mut bcast_response = false;
            for i in 0..5 {
                log::debug!("Channel broadcast try {}/5", i+1);
                let mut broadcast = [self.ecu_id, 0xC0, 0x00, 0x10, 0x00, 0x00, T::PROTOCOL_ID];
                broadcast[4] = (*rx & 0xFF) as u8;
                broadcast[5] = ((*rx >> 8) & 0x0F) as u8;
                let frame = CanFrame::new(0x200, &broadcast, false);
                can.write_packets(vec![frame], 100)?;
                let rx_id = 0x200 + self.ecu_id as u32;
                std::thread::sleep(Duration::from_millis(20));
                if let Ok(packets) = can.read_packets(1000, 100) {
                    if let Some(bcast_resp) = packets.iter().find(|x| x.get_address() == rx_id) {
                        log::debug!("Got response!: {bcast_resp:02X?}");
                        let data = bcast_resp.get_data();
                        if data[1] == 0xD0 {
                            // ECU is OK
                            let can_tx_id = (((data[5] & 0x0F) as u16) << 8) | data[4] as u16;
                            if can_tx_id != *tx {
                                log::warn!("CAN Tx ID negotiated (0x{:04X}) is not the same as user requested (0x{:04X}). Overriding", can_tx_id, *tx)
                            }
                            *tx = can_tx_id;
                            bcast_response = true;
                        } else if data[1] == 0xD6 {
                            return Err(ChannelError::Other("ECU Reject. Application type not supported".into()))
                        } else if data[1] == 0xD7 {
                            return Err(ChannelError::Other("ECU Reject. Application type temporarily not supported".into()))
                        } else if data[1] == 0xD8 {
                            return Err(ChannelError::Other("ECU Reject. Temporarily no resources are free".into()))
                        } else {
                            // Negative response
                            return Err(ChannelError::UnsupportedRequest)
                        }
                        break;
                    }
                }
            }
            if !bcast_response {
                return Err(ChannelError::ReadTimeout)
            }
            // Step 2 - Setup communication parameters
            let config_req = &[
                0xA0,
                self.settings.bs,
                make_timing_byte(self.settings.ack_timeout),
                0xFF,
                make_timing_byte(self.settings.inter_packet_spacing_ms),
                0xFF
            ];
            let frame = CanFrame::new((*tx) as u32, config_req, false);
            log::debug!("Sending configuration packet");
            can.clear_rx_buffer()?;
            can.write_packets(vec![frame], 100)?;
            if let Some(config_resp) = can.read_packets(1000, 100)?.iter().find(|x| x.get_address() == (*rx) as u32) {
                let data = config_resp.get_data();
                if data[0] == 0xA1 {
                    let bs = data[1];
                    let ack_timeout = decode_timing_byte(data[2]);
                    let st_min = decode_timing_byte(data[4]);
                    log::debug!("ECU configuration reply: BS: {}, Ack timeout: {:?}, ST_MIN: {:?}", bs, ack_timeout, st_min);
                    self.start_background_thread(bs, ack_timeout, st_min, self.settings);
                    Ok(())
                } else {
                    // ??
                    Err(ChannelError::ReadTimeout)
                }
            } else {
                Err(ChannelError::ReadTimeout)
            }
        } else {
            Err(ChannelError::ConfigurationError)
        }
    }

    fn close(&mut self) -> ChannelResult<()> {
        if let Some((tx, _)) = self.tx_rx_ids {
            let mut can_channel = match std::mem::replace(&mut self.can, ChannelState::Uninit) {
                ChannelState::Uninit => unreachable!(),
                ChannelState::Ready(can_channel) => can_channel,
                ChannelState::Running { handle, is_running, .. } => {
                    is_running.store(false, std::sync::atomic::Ordering::Relaxed);
                    handle.join().unwrap()
                },
            };

            // Send close packet
            log::debug!("Sending Channel close");
            let frame = CanFrame::new(tx as u32, &[0xA8], false);
            let _ = can_channel.write_packets(vec![frame], 0);
            let res = can_channel.close();
            self.can = ChannelState::Ready(can_channel);
            res?;
        }
        Ok(())
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        let valid_range = 0..=0x7FFu32;
        if !valid_range.contains(&send) || !valid_range.contains(&recv) || send == recv {
            Err(ChannelError::ConfigurationError)
        } else {
            self.tx_rx_ids = Some((send as u16, recv as u16));
            Ok(())
        }
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        if let ChannelState::Running { read_rx, .. } = &mut self.can {
            let data = read_rx.lock().unwrap().recv_timeout(Duration::from_millis(timeout_ms as u64))
                .map_err(|e| {println!("{e:?}"); ChannelError::ReadTimeout})?;
            T::decode_data(&data)
        } else {
            Err(ChannelError::ConfigurationError)
        }
    }

    fn write_bytes(
        &mut self,
        _addr: u32,
        _ext_id: Option<u8>,
        buffer: &[u8],
        _timeout_ms: u32,
    ) -> ChannelResult<()> {
        if let ChannelState::Running { write_tx, write_tx_resp, .. } = &mut self.can {
            write_tx.send(T::encode_data(buffer))?;
            write_tx_resp.lock().unwrap().recv().unwrap()
        } else {
            // Channel isn't open?
            Err(ChannelError::ConfigurationError)
        }
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        if let ChannelState::Running { read_rx, .. } = &mut self.can {
            while read_rx.lock().unwrap().try_recv().is_ok(){}
            Ok(())
        } else {
            Err(ChannelError::ConfigurationError)
        }
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }
}