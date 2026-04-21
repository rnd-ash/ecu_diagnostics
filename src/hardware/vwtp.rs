//! VW Transport protocol 2
//! 
//! This piggy-backs off an open CAN Channel, to provide a PayloadChannel
//! that can be used by the Diagnostic server

use std::{sync::{Arc, Mutex, atomic::AtomicBool, mpsc}, thread::JoinHandle, time::{Duration, Instant}};

use crate::channel::{CanChannel, CanFrame, ChannelError, ChannelResult, Packet, PacketChannel, PayloadChannel, VwTp2Settings};

#[derive(Clone)]
pub enum VwChannelCmd {
    SetIds(u16, u16),
    Open,
    Close,
    Write(Vec<u8>)
}

pub enum ChannelState {
    None,
    Init(u16, u16),
    Open(u16, u16)
}

pub struct VwTp2 {
    ecu_id: u8,
    settings: VwTp2Settings,
    tx_rx_ids: Option<(u16, u16)>,
    can: Arc<Mutex<Box<dyn CanChannel>>>,
    running: Arc<AtomicBool>
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

impl VwTp2 {
    pub fn new(can: Box<dyn CanChannel>, ecu_id: u8, settings: VwTp2Settings) -> ChannelResult<Self> {
        if can.is_open() {
            Err(ChannelError::ConfigurationError)
        } else {
            Ok(Self {
                ecu_id,
                settings,
                tx_rx_ids: None,
                can: Arc::new(Mutex::new(can)),
                running: Arc::new(AtomicBool::new(false))
            })
        }
    }

    pub fn start_background_thread(&mut self, bs: u8, ack_timeout: Duration, inter_packet_ms: Duration) {
        self.running.store(true, std::sync::atomic::Ordering::Relaxed);
        let is_running = self.running.clone();
        let can = self.can.clone();
        let (tx_id, rx_id) = self.tx_rx_ids.unwrap();
        std::thread::spawn(move || {
            log::debug!("Keep alive thread started");
            let keep_alive = CanFrame::new(tx_id as u32, &[0xA3], false);
            let mut last_ping = Instant::now();
            while is_running.load(std::sync::atomic::Ordering::Relaxed) {
                let recorded_packets = if let Ok(mut lock) = can.try_lock() {
                    if last_ping.elapsed().as_millis() > 1000 {
                        log::debug!("Channel ping");
                        let _ = lock.write_packets(vec![keep_alive.clone()], 0);
                        last_ping = Instant::now();
                    }
                    lock.read_packets(1000, 0).unwrap_or_default()
                } else {
                    Vec::default()
                };
                for packet in recorded_packets.iter().filter(|x| x.get_address() == rx_id as u32) {
                    println!("ECU Packet: {packet:02X?}");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            log::debug!("Keep alive thread killed");
            is_running.store(false, std::sync::atomic::Ordering::Relaxed);
        });
    }
}

impl PayloadChannel for VwTp2 {
    fn open(&mut self) -> ChannelResult<()> {
        if let Some((tx, rx)) = self.tx_rx_ids.as_mut() {
            self.can.open()?;
            self.can.clear_rx_buffer()?;
            self.can.clear_tx_buffer()?;
            // Step 1 - Broadcast - See if the ECU replies to us
            let mut bcast_response = false;
            for i in 0..5 {
                log::debug!("Channel broadcast try {}/5", i+1);
                let mut broadcast = [self.ecu_id, 0xC0, 0x00, 0x10, 0x00, 0x00, 0x01];
                broadcast[4] = (*rx & 0xFF) as u8;
                broadcast[5] = ((*rx >> 8) & 0x0F) as u8;
                let frame = CanFrame::new(0x200, &broadcast, false);
                self.can.write_packets(vec![frame], 100)?;
                let rx_id = 0x200 + self.ecu_id as u32;
                std::thread::sleep(Duration::from_millis(20));
                if let Ok(packets) = self.can.read_packets(1000, 10) {
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
            self.can.clear_rx_buffer()?;
            self.can.write_packets(vec![frame], 100)?;
            if let Some(config_resp) = self.can.read_packets(100, 20)?.iter().find(|x| x.get_address() == (*rx) as u32) {
                let data = config_resp.get_data();
                if data[0] == 0xA1 {
                    let bs = data[1];
                    let ack_timeout = decode_timing_byte(data[2]);
                    let st_min = decode_timing_byte(data[4]);
                    log::debug!("ECU configuration reply: BS: {}, Ack timeout: {:?}, ST_MIN: {:?}", bs, ack_timeout, st_min);
                    self.start_background_thread(bs, ack_timeout, st_min);
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
        if let Some((tx, rx)) = self.tx_rx_ids {
            self.running.store(false, std::sync::atomic::Ordering::Relaxed);
            // Send close packet
            log::debug!("Sending Channel close");
            let frame = CanFrame::new(tx as u32, &[0xA8], false);
            let _ = self.can.write_packets(vec![frame], 0);
        }
        self.can.close()?;
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
        todo!()
    }

    fn write_bytes(
        &mut self,
        _addr: u32,
        _ext_id: Option<u8>,
        buffer: &[u8],
        _timeout_ms: u32,
    ) -> ChannelResult<()> {
        Ok(())
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }
}

#[cfg(test)]
pub mod vw_tp_test {
    use std::time::Duration;

    use crate::{channel::{PayloadChannel, VwTp2Settings}, hardware::{Hardware, HardwareScanner, socketcan::SocketCanScanner, vwtp::VwTp2}};

    #[test]
    pub fn test_vwtp() {
        env_logger::init();
        let mut hw = SocketCanScanner::new().open_device_by_name("can0").unwrap();
        let mut can = hw.create_can_channel().unwrap();
        let vw_settings = VwTp2Settings::default();
        let mut vwtp = VwTp2::new(can, 0x0A, vw_settings).unwrap();
        vwtp.set_ids(0x300, 0x308).unwrap();
        vwtp.open().unwrap();
        std::thread::sleep(Duration::from_millis(5000));
        vwtp.close().unwrap();
        //vwtp.write_bytes(0x300, None, &[0x10, 0x92], 1000).unwrap();
        //println!("{:?}", vwtp.read_bytes(1000));
    }
}