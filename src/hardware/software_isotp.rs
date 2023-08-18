//! Software ISOTP layer

use std::{sync::{atomic::{AtomicBool, Ordering, AtomicU32}, Arc, mpsc}, collections::VecDeque, time::{Instant, Duration}};

use log::debug;

use crate::channel::{PayloadChannel, IsoTPChannel, CanFrame, PacketChannel, CanChannel, IsoTPSettings, ChannelResult, ChannelError, Packet};

#[derive(Debug, Clone)]
/// Software ISOTP layer.
/// This is useful for certain hardware layers that might not
/// natively support ISO-TP, but support CAN
pub struct SoftwareIsoTpChannel {
    running: Arc<AtomicBool>,
    can_msg_sender: mpsc::Sender<CanMessage>,
    isotp_msg_sender: mpsc::Sender<IsoTpMessage>,
}

#[derive(Debug)]
enum IsoTpMessage {
    Open(mpsc::Sender<ChannelResult<()>>),
    Close(mpsc::Sender<ChannelResult<()>>),
    SetIds(u32, u32, mpsc::Sender<ChannelResult<()>>),
    ReadBytes(u32,  mpsc::Sender<ChannelResult<Vec<u8>>>),
    WriteBytes(u32, Option<u8>, Vec<u8>, u32, mpsc::Sender<ChannelResult<()>>),
    ClearRxBuffer(mpsc::Sender<ChannelResult<()>>),
    ClearTxBuffer(mpsc::Sender<ChannelResult<()>>),
    SetCfg(IsoTPSettings, mpsc::Sender<ChannelResult<()>>)
}

#[derive(Debug)]
enum CanMessage {
    Open(mpsc::Sender<ChannelResult<()>>),
    Close(mpsc::Sender<ChannelResult<()>>),
    Configure(u32, bool, mpsc::Sender<ChannelResult<()>>),
    ReadFrames(usize, u32, mpsc::Sender<ChannelResult<Vec<CanFrame>>>),
    WriteFrames(u32, Vec<CanFrame>, mpsc::Sender<ChannelResult<()>>),
    ClearRxBuffer(mpsc::Sender<ChannelResult<()>>),
}

unsafe impl Sync for SoftwareIsoTpChannel{}
unsafe impl Send for SoftwareIsoTpChannel{}

impl SoftwareIsoTpChannel {
    /// Returns this as a ISOTP channel
    pub fn as_isotp_channel(&self) -> Box<dyn IsoTPChannel> {
        Box::new(self.clone())
    }

    /// Returns this as a CAN channel
    pub fn as_can_channel(&self) -> Box<dyn CanChannel> {
        Box::new(self.clone())
    }

    /// Creates a new Software ISOTP channel
    pub fn new(mut channel: Box<dyn CanChannel>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_c = running.clone();
        let running_cc = running.clone();

        let baud: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
        let baud_c: Arc<AtomicU32> = baud.clone();
        let ext_can: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let ext_can_c = ext_can.clone();

        let isotp_listen_id = Arc::new(AtomicU32::new(0));
        let isotp_list_id_c = isotp_listen_id.clone();

        let mut can_open = Arc::new(AtomicBool::new(false));
        let mut can_open_c = can_open.clone();

        let (can_to_isotp_rx_frame_tx, can_to_isotp_rx_frame_rx) = mpsc::channel::<CanFrame>();

        let (isotp_msg_sender, isotp_msg_receiver) = mpsc::channel::<IsoTpMessage>();
        let (can_msg_sender, can_msg_receiver) = mpsc::channel::<CanMessage>();
        let can_msg_sender_isotp = can_msg_sender.clone();

        std::thread::spawn(move|| {

            // Rx information
            let mut rx_bytes: Vec<u8> = Vec::new();
            let mut rx_bs_now = 0; // Current block ID
            let mut rx_size: u16 = 0; // Expected Rx size
            let mut rx_done: bool = false;
            let mut bg_rx_receiver: Option<mpsc::Sender<ChannelResult<Vec<u8>>>> = None;
            let mut rx_start_time = Instant::now();
            let mut rx_timeout = 0;


            let mut isotp_settings: Option<IsoTPSettings> = None;
            let mut default_tx_addr = 0;
            let mut isotp_running = false;
            while running_c.load(Ordering::Relaxed) {
                if let Ok(msg) = isotp_msg_receiver.try_recv() {
                    debug!("ISOTP request msg: {msg:02X?}");
                    match msg {
                        IsoTpMessage::Open(sender_resp) => {
                            let res = if can_open_c.load(Ordering::Relaxed) {
                                isotp_running = true;
                                Ok(())
                            } else {
                                let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
                                let _ = can_msg_sender_isotp.send(CanMessage::Open(tx));
                                let resp = rx.recv().unwrap();
                                if resp.is_ok() {
                                    isotp_running = true;
                                }
                                resp
                            };
                            let _ = sender_resp.send(res);
                        },
                        IsoTpMessage::Close(sender_resp) => {
                            // Do Not kill the CAN channel on close!
                            isotp_running = false;
                            let _ = sender_resp.send(Ok(()));
                        },
                        IsoTpMessage::SetIds(send, recv, sender_resp) => {
                            default_tx_addr = send;
                            isotp_listen_id.store(recv, Ordering::Relaxed);
                            let _ = sender_resp.send(Ok(()));
                        },
                        IsoTpMessage::ReadBytes(timeout_ms, sender_resp) => {
                            if rx_done {
                                debug!("RX done!: {:02X?}", rx_bytes);
                                let _ = sender_resp.send(Ok(rx_bytes.clone()));
                                rx_bytes.clear();
                                rx_bs_now = 0;
                                rx_size = 0;
                                rx_done = false;
                            } else if timeout_ms == 0 {
                                // Non blocking - No data in buffer
                                let _ = sender_resp.send(Err(ChannelError::BufferEmpty));
                            } else {
                                // Blocking - No data in buffer
                                rx_start_time = Instant::now();
                                rx_timeout = timeout_ms;
                                bg_rx_receiver = Some(sender_resp);
                            }
                        },
                        IsoTpMessage::WriteBytes(send_id, ext_id, data, timeout_ms, sender_resp) => {
                            let res = match isotp_settings.ok_or(ChannelError::ConfigurationError) {
                                Ok(cfg) => {
                                    let mut max_len = 7;
                                    if ext_id.is_some() {
                                        max_len -= 1;
                                    }
                                    if data.len() <= max_len {
                                        let mut data_len = 8;
                                        if !cfg.pad_frame {
                                            data_len = max_len+1; // +1 for protocol byte
                                        }
                                        // 1 time send
                                        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
                                        let mut tx_data: Vec<u8> = vec![0xCC; data_len];
                                        let mut c_pos = 0;
                                        if let Some(id) = ext_id {
                                            tx_data[c_pos] = id;
                                            c_pos+=1;
                                        }
                                        tx_data[c_pos] = data.len() as u8;
                                        c_pos += 1;
                                        tx_data[c_pos..c_pos+data.len()].copy_from_slice(&data);
                                        let f = CanFrame::new(send_id, &tx_data, cfg.can_use_ext_addr);
                                        can_msg_sender_isotp.send(CanMessage::WriteFrames(0, vec![f], tx));

                                        rx.recv().unwrap()
                                    } else {
                                        // Multi frame Tx
                                        
                                        Err(ChannelError::UnsupportedRequest)
                                    }
                                },
                                Err(e) => Err(e)
                            };
                            sender_resp.send(res);
                        },
                        IsoTpMessage::ClearRxBuffer(sender_resp) => {
                            while can_to_isotp_rx_frame_rx.try_recv().is_ok(){}
                            let _ = sender_resp.send(Ok(()));
                        },
                        IsoTpMessage::ClearTxBuffer(sender_resp) => {
                            let _ = sender_resp.send(Ok(()));
                        },
                        IsoTpMessage::SetCfg(cfg, sender_resp) => {
                            let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
                            let _ = can_msg_sender_isotp.send(CanMessage::Configure(cfg.can_speed, cfg.can_use_ext_addr, tx));
                            let resp = rx.recv().unwrap();
                            if resp.is_ok() {
                                isotp_settings = Some(cfg);
                            }
                            sender_resp.send(resp);
                        },
                    }
                }
                // Frame from an ECU 
                if let Ok(frame) = can_to_isotp_rx_frame_rx.try_recv() {
                    if let Some(cfg) = isotp_settings {
                        let data = frame.get_data();
                        let pci_byte_idx = match cfg.extended_addresses.is_some() {
                            true => 1,
                            false => 0,
                        };
                        match data.get(pci_byte_idx) {
                            Some(pci) => {
                                match pci & 0xF0 {
                                    0x00 => {
                                        log::debug!("ISOTP One frame {data:02X?}");
                                        let len = *pci as usize;
                                        if len <= 7 - pci_byte_idx {
                                            rx_bytes = data[pci_byte_idx+1..pci_byte_idx + len+1].to_vec();
                                            rx_done = true;
                                        } else {
                                            log::error!("Invalid ISOTP CAN frame! {frame:?}");
                                        }
                                    },
                                    0x10 => { // Start of multi frame
                                        log::debug!("ISOTP Start frame {data:02X?}")
                                    },
                                    0x20 => { // Continuation of multi frame
                                        log::debug!("ISOTP continue frame {data:02X?}")
                                    },
                                    0x30 => { // Flow control
                                        log::debug!("ISOTP Flow control {data:02X?}")
                                    }
                                    _ => {
                                        log::error!("Invalid ISOTP CAN frame! {frame:?}");
                                    }
                                }
                            },
                            None => {
                                log::error!("ISOTP CAN frame too short! {frame:?}");
                            },
                        }
                    }
                }
                
                // Check for Rx status
                if bg_rx_receiver.is_some() {
                    if rx_done {
                        // Done!
                        bg_rx_receiver.take().unwrap().send(Ok(rx_bytes.clone()));
                        rx_done = false;
                        rx_bytes.clear();
                        rx_bs_now = 0;
                        rx_size = 0;
                        rx_done = false;
                    } else if rx_start_time.elapsed().as_millis() >= rx_timeout as u128 {
                        bg_rx_receiver.take().unwrap().send(Err(ChannelError::ReadTimeout));
                        rx_done = false;
                        rx_bytes.clear();
                        rx_bs_now = 0;
                        rx_size = 0;
                        rx_done = false;
                    }
                }
            }
        });

        // CAN channel dispatcher - Sends and receives CAN frames from raw interface
        std::thread::spawn(move|| {
            let mut can_cfg: Option<(u32, bool)> = None;
            let mut can_queue: VecDeque<CanFrame> = VecDeque::new();
            let mut is_reading = false;
            let mut reading_length: usize = 0;
            let mut reading_timeout: u32 = 0;
            let mut reading_start = Instant::now();
            let mut res_read: Vec<CanFrame> = vec![];
            let mut sender_read_res: Option<mpsc::Sender<ChannelResult<Vec<CanFrame>>>> = None;
            while running_cc.load(Ordering::Relaxed) {
                if let Ok(msg) = can_msg_receiver.try_recv() {
                    debug!("CAN request msg: {msg:02X?}");
                    match msg {
                        CanMessage::Open(resp_sender) => {
                            let res = channel.open();
                            if res.is_ok() {
                                can_open.store(true, Ordering::Relaxed);
                            }
                            let _ = resp_sender.send(res);
                        },
                        CanMessage::Close(resp_sender) => {
                            let res = channel.close();
                            if res.is_ok() {
                                can_open.store(false, Ordering::Relaxed);
                            }
                            let _ = resp_sender.send(res);
                        },
                        CanMessage::Configure(baud, ext, resp_sender) => {
                            // If configurations are the same, then we can allow this 
                            if let Some(current_cfg) = can_cfg {
                                if current_cfg == (baud, ext) {
                                    resp_sender.send(Ok(()));
                                    continue;
                                }
                            }
                            let _ = if can_open.load(Ordering::Relaxed) {
                                resp_sender.send(Err(ChannelError::ConfigurationError))
                            } else {
                                let res = channel.set_can_cfg(baud, ext);
                                if res.is_ok() {
                                    can_cfg = Some((baud, ext));
                                }
                                resp_sender.send(res)
                            };
                        },
                        CanMessage::ReadFrames(max, timeout, resp_sender) => {
                            res_read.clear();
                            // Read what we have already
                            while let Some(f) = can_queue.pop_front() {
                                res_read.push(f);
                                if res_read.len() == max {
                                    break;
                                }
                            }
                            if timeout == 0 || res_read.len() == max {
                                let _ = resp_sender.send(Ok(res_read.clone()));
                            } else {
                                // Blocking, reading in background
                                is_reading = true;
                                reading_start = Instant::now();
                                reading_timeout = timeout;
                                reading_length = max;
                                sender_read_res = Some(resp_sender)
                            }
                        },
                        CanMessage::WriteFrames(_timeout, f, resp_sender) => {
                            // TODO timeout
                            let _ = resp_sender.send(channel.write_packets(f, 0));
                        },
                        CanMessage::ClearRxBuffer(resp_sender) => {
                            can_queue.clear();
                            let _ = resp_sender.send(Ok(())); // Don't clear Hardware buffer, since this is also in use for ISOTP
                        }
                    }
                }
                if can_open.load(Ordering::Relaxed) {
                    if let Ok(packets) = channel.read_packets(100, 0) {
                        let read_id = isotp_list_id_c.load(Ordering::Relaxed);
                        for frame in packets {
                            if read_id == frame.get_address() {
                                debug!("Found CAN Frame to send to ISOTP: {frame:02X?}");
                                let _ = can_to_isotp_rx_frame_tx.send(frame.clone());
                            }
                            can_queue.push_back(frame);
                        }
                    }
                    if is_reading {
                        while let Some(f) = can_queue.pop_front() {
                            res_read.push(f);
                            if res_read.len() == reading_length {
                                break;
                            }
                        }
                        if reading_length == res_read.len() {
                            // Target length reached
                            let _ = sender_read_res.take().unwrap().send(Ok(res_read.clone()));
                            is_reading = false;
                        } else if reading_start.elapsed().as_millis() > reading_timeout as u128 {
                            // Timeout reached
                            let _ = sender_read_res.take().unwrap().send(Err(ChannelError::ReadTimeout));
                            is_reading = false;
                        }
                    }
                } else {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        });

        Self {
            running,
            can_msg_sender: can_msg_sender,
            isotp_msg_sender: isotp_msg_sender
        }
    }
}

impl PayloadChannel for SoftwareIsoTpChannel {
    fn open(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender.send(IsoTpMessage::Open(tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn close(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender.send(IsoTpMessage::Close(tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender.send(IsoTpMessage::SetIds(send, recv, tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        let (tx, rx) = mpsc::channel::<ChannelResult<Vec<u8>>>();
        self.isotp_msg_sender.send(IsoTpMessage::ReadBytes(timeout_ms, tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn write_bytes(
        &mut self,
        addr: u32,
        ext_id: Option<u8>,
        buffer: &[u8],
        timeout_ms: u32,
    ) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender.send(IsoTpMessage::WriteBytes(addr, ext_id, buffer.to_vec(), timeout_ms, tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender.send(IsoTpMessage::ClearRxBuffer(tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender.send(IsoTpMessage::ClearTxBuffer(tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }
}

impl IsoTPChannel for SoftwareIsoTpChannel {
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender.send(IsoTpMessage::SetCfg(cfg, tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }
}

impl PacketChannel<CanFrame> for SoftwareIsoTpChannel {
    fn open(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender.send(CanMessage::Open(tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn close(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender.send(CanMessage::Close(tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, timeout_ms: u32) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender.send(CanMessage::WriteFrames(timeout_ms, packets, tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> ChannelResult<Vec<CanFrame>> {
        let (tx, rx) = mpsc::channel::<ChannelResult<Vec<CanFrame>>>();
        self.can_msg_sender.send(CanMessage::ReadFrames(max, timeout_ms, tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender.send(CanMessage::ClearRxBuffer(tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }
}

impl CanChannel for SoftwareIsoTpChannel {
    fn set_can_cfg(&mut self, baud: u32, use_extended: bool) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender.send(CanMessage::Configure(baud, use_extended, tx)).map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }
}