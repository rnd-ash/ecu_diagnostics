//! Software ISOTP layer
use std::{
    cmp::min,
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};

use log::debug;

use crate::channel::{
    CanChannel, CanFrame, ChannelError, ChannelResult, IsoTPChannel, IsoTPSettings, Packet,
    PacketChannel, PayloadChannel,
};

#[derive(Debug, Clone)]
/// Software ISOTP layer.
/// This is useful for certain hardware layers that might not
/// natively support ISO-TP, but support CAN
pub struct SoftwareIsoTpChannel {
    running: Arc<AtomicBool>,
    can_msg_sender: mpsc::Sender<CanMessage>,
    isotp_msg_sender: mpsc::Sender<IsoTpMessage>,
}

impl Drop for SoftwareIsoTpChannel {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

struct IsoTpRxMemory {
    pub last_rx_time: Instant,
    pub completed: bool,
    pub receiving: bool,
    pub bs: u8,
    pub frames_received: usize,
    pub data: Vec<u8>,
    pub max_size: usize,
}

impl Default for IsoTpRxMemory {
    fn default() -> Self {
        Self {
            last_rx_time: Instant::now(),
            completed: false,
            receiving: false,
            bs: 0,
            frames_received: 0,
            data: vec![],
            max_size: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum IsoTpRxAction {
    None,
    Completed,
    SendFC,
}

impl IsoTpRxMemory {
    pub fn reset(&mut self) {
        self.data.clear();
        self.completed = false;
        self.receiving = false;
        self.frames_received = 0;
    }

    pub fn add_single_frame(&mut self, s: &[u8]) {
        self.completed = true;
        self.receiving = false;
        let len = s[0] as usize;
        self.data = s[1..1 + len].to_vec();
    }

    pub fn add_start_frame(&mut self, s: &[u8]) {
        self.max_size = ((((s[0] & 0x0F) as u16) << 8) | (s[1] as u16)) as usize;
        self.receiving = true;
        self.frames_received = 0;
        self.data.extend_from_slice(&s[2..]);
        self.last_rx_time = Instant::now();
    }

    // Returns true if Rx is done!
    pub fn add_continuous_frame(&mut self, s: &[u8]) -> IsoTpRxAction {
        let max_copy = min(self.max_size - self.data.len(), 7);
        self.data.extend_from_slice(&s[1..1 + max_copy]);
        self.frames_received += 1;
        self.last_rx_time = Instant::now();
        if self.data.len() == self.max_size {
            self.completed = true;
            IsoTpRxAction::Completed
        } else if self.frames_received == self.bs as usize && self.bs != 0 {
            IsoTpRxAction::SendFC
        } else {
            IsoTpRxAction::None
        }
    }
}

struct IsoTpTxMemory {
    pub addr: u32,
    pub completed: bool,
    pub transmitting: bool,
    pub awaiting_fc: bool,
    pub last_tx_time: Instant,
    pub frames_txed: usize,
    pub data: Vec<u8>,
    pub current_pos: usize,
    pub current_pci: u8,
    // Set by receiving ECU
    pub fc_bs: u8,
    // Set by receiving ECU
    pub fc_stmin: u8,
}

impl Default for IsoTpTxMemory {
    fn default() -> Self {
        Self {
            frames_txed: 0,
            addr: 0,
            completed: false,
            transmitting: false,
            awaiting_fc: false,
            last_tx_time: Instant::now(),
            data: vec![],
            current_pos: 0,
            current_pci: 0x21,
            fc_bs: 0,
            fc_stmin: 0,
        }
    }
}

impl IsoTpTxMemory {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn on_flow_control(&mut self, data: &[u8]) {
        self.fc_bs = data[1];
        self.fc_stmin = data[2];
        self.awaiting_fc = false;
        self.last_tx_time = Instant::now();
        self.frames_txed = 0;
    }

    pub fn get_start_frame(&mut self) -> [u8; 8] {
        let mut tx = [0; 8];
        tx[0] = 0x10 | ((self.data.len() >> 8) & 0x0F) as u8;
        tx[1] = (self.data.len() & 0xFF) as u8;
        tx[2..8].copy_from_slice(&self.data[0..6]);
        self.current_pos = 6;
        self.awaiting_fc = true;
        self.completed = false;
        self.current_pci = 0x21;
        self.last_tx_time = Instant::now();
        self.transmitting = true;
        self.frames_txed = 0;
        tx
    }

    pub fn on_update(&mut self, timeout: u32, pad_frame: bool) -> Option<ChannelResult<Vec<u8>>> {
        if self.completed {
            return None;
        }

        let mut t_out = timeout;
        if timeout == 0 {
            t_out = 1000
        }
        if self.transmitting {
            // Timeout for awaiting FC
            if self.awaiting_fc && self.last_tx_time.elapsed().as_millis() > (t_out * 2) as u128 {
                log::error!(
                    "Awaiting FC timed out. {timeout} - {}",
                    self.last_tx_time.elapsed().as_millis()
                );
                Some(Err(ChannelError::WriteTimeout))
            } else if self.last_tx_time.elapsed().as_millis() >= self.fc_stmin as u128
                || self.fc_stmin == 0
            {
                // We can transmit

                let mut tx_data = vec![];
                tx_data.push(self.current_pci);

                let max_data = min(7, self.data.len() - self.current_pos);
                tx_data
                    .extend_from_slice(&self.data[self.current_pos..self.current_pos + max_data]);
                if pad_frame {
                    tx_data.resize(8, 0xCC);
                }
                self.current_pos += max_data;

                if self.fc_bs != 0 && self.frames_txed > self.fc_bs as usize {
                    // Await flow control after this update!
                    log::debug!("Awaiting FC");
                    self.awaiting_fc = true;
                    self.frames_txed = 0;
                }

                if self.current_pos >= self.data.len() {
                    log::debug!("Tx done!");
                    self.completed = true;
                }

                self.frames_txed += 1;
                self.current_pci += 1;
                if self.current_pci == 0x30 {
                    self.current_pci = 0x20;
                }
                Some(Ok(tx_data))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
enum IsoTpMessage {
    Open(mpsc::Sender<ChannelResult<()>>),
    Close(mpsc::Sender<ChannelResult<()>>),
    SetIds(u32, u32, mpsc::Sender<ChannelResult<()>>),
    ReadBytes(u32, mpsc::Sender<ChannelResult<Vec<u8>>>),
    WriteBytes(
        u32,
        Option<u8>,
        Vec<u8>,
        u32,
        mpsc::Sender<ChannelResult<()>>,
    ),
    ClearRxBuffer(mpsc::Sender<ChannelResult<()>>),
    ClearTxBuffer(mpsc::Sender<ChannelResult<()>>),
    SetCfg(IsoTPSettings, mpsc::Sender<ChannelResult<()>>),
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

unsafe impl Sync for SoftwareIsoTpChannel {}
unsafe impl Send for SoftwareIsoTpChannel {}

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

        let _baud: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
        let _ext_can: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let isotp_listen_id = Arc::new(AtomicU32::new(0));
        let isotp_list_id_c = isotp_listen_id.clone();

        let can_open = Arc::new(AtomicBool::new(false));
        let can_open_c = can_open.clone();

        let (can_to_isotp_rx_frame_tx, can_to_isotp_rx_frame_rx) = mpsc::channel::<CanFrame>();

        let (isotp_msg_sender, isotp_msg_receiver) = mpsc::channel::<IsoTpMessage>();
        let (can_msg_sender, can_msg_receiver) = mpsc::channel::<CanMessage>();
        let can_msg_sender_isotp = can_msg_sender.clone();

        std::thread::spawn(move || {
            let mut rx_memory = IsoTpRxMemory::default();
            let mut bg_rx_receiver: Option<mpsc::Sender<ChannelResult<Vec<u8>>>> = None;
            let mut rx_timeout = 0;

            let mut tx_memory = IsoTpTxMemory::default();
            let mut bg_tx_receiver: Option<mpsc::Sender<ChannelResult<()>>> = None;
            let mut tx_timeout = 0;

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
                        }
                        IsoTpMessage::Close(sender_resp) => {
                            // Do Not kill the CAN channel on close!
                            isotp_running = false;
                            let _ = sender_resp.send(Ok(()));
                        }
                        IsoTpMessage::SetIds(send, recv, sender_resp) => {
                            default_tx_addr = send;
                            isotp_listen_id.store(recv, Ordering::Relaxed);
                            let _ = sender_resp.send(Ok(()));
                        }
                        IsoTpMessage::ReadBytes(timeout_ms, sender_resp) => {
                            if rx_memory.completed {
                                debug!("RX done!: {:02X?}", rx_memory.data);
                                let _ = sender_resp.send(Ok(rx_memory.data.clone()));
                                rx_memory.reset();
                            } else if timeout_ms == 0 {
                                // Non blocking - No data in buffer
                                let _ = sender_resp.send(Err(ChannelError::BufferEmpty));
                            } else {
                                // Blocking - No data in buffer
                                rx_memory.last_rx_time = Instant::now();
                                rx_timeout = timeout_ms;
                                bg_rx_receiver = Some(sender_resp);
                            }
                        }
                        IsoTpMessage::WriteBytes(
                            send_id,
                            ext_id,
                            data,
                            timeout_ms,
                            sender_resp,
                        ) => {
                            if ext_id.is_some() {
                                // TODO Ext ID tx
                                let _ = sender_resp.send(Err(ChannelError::UnsupportedRequest));
                            } else {
                                let res = match isotp_settings
                                    .ok_or(ChannelError::ConfigurationError)
                                {
                                    Ok(cfg) => {
                                        if data.len() <= 7 {
                                            let mut data_len = data.len() + 1;
                                            if cfg.pad_frame {
                                                data_len = 8;
                                            }
                                            // 1 time send
                                            let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
                                            let mut tx_data: Vec<u8> = vec![0xCC; data_len]; // Fill with padding byte
                                            tx_data[0] = data.len() as u8;
                                            tx_data[1..1 + data.len()].copy_from_slice(&data);
                                            let f = CanFrame::new(
                                                send_id,
                                                &tx_data,
                                                cfg.can_use_ext_addr,
                                            );
                                            let _ = can_msg_sender_isotp
                                                .send(CanMessage::WriteFrames(0, vec![f], tx));
                                            rx.recv().unwrap()
                                        } else {
                                            // Multi frame Tx
                                            if data.len() > 4095 {
                                                // Data too large for ISO-TP
                                                Err(ChannelError::UnsupportedRequest)
                                            } else if tx_memory.transmitting {
                                                Err(ChannelError::BufferFull)
                                            } else {
                                                tx_memory.reset();
                                                tx_memory.data = data.clone();
                                                tx_timeout = timeout_ms;
                                                tx_memory.addr = send_id;
                                                let tx_data = tx_memory.get_start_frame();
                                                // Send the ISO-TP start frame
                                                let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
                                                let f = CanFrame::new(
                                                    send_id,
                                                    &tx_data,
                                                    cfg.can_use_ext_addr,
                                                );
                                                let _ = can_msg_sender_isotp
                                                    .send(CanMessage::WriteFrames(0, vec![f], tx));
                                                rx.recv().unwrap()
                                            }
                                        }
                                    }
                                    Err(e) => Err(e),
                                };
                                let _ = sender_resp.send(res);
                            }
                        }
                        IsoTpMessage::ClearRxBuffer(sender_resp) => {
                            while can_to_isotp_rx_frame_rx.try_recv().is_ok() {}
                            rx_memory.reset();
                            let _ = sender_resp.send(Ok(()));
                        }
                        IsoTpMessage::ClearTxBuffer(sender_resp) => {
                            tx_memory.reset();
                            let _ = sender_resp.send(Ok(()));
                        }
                        IsoTpMessage::SetCfg(cfg, sender_resp) => {
                            let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
                            let _ = can_msg_sender_isotp.send(CanMessage::Configure(
                                cfg.can_speed,
                                cfg.can_use_ext_addr,
                                tx,
                            ));
                            let resp = rx.recv().unwrap();
                            if resp.is_ok() {
                                isotp_settings = Some(cfg);
                            }
                            let _ = sender_resp.send(resp);
                        }
                    }
                }
                // Frame from an ECU
                if let Ok(frame) = can_to_isotp_rx_frame_rx.try_recv() {
                    if let Some(cfg) = isotp_settings {
                        let data = frame.get_data();
                        let pci_byte_idx = 0; // TODO for EXT ID Rx
                        match data.get(pci_byte_idx) {
                            Some(pci) => {
                                match pci & 0xF0 {
                                    0x00 => {
                                        log::debug!("ISOTP One frame {data:02X?}");
                                        rx_memory.add_single_frame(data);
                                    }
                                    0x10 => {
                                        // Start of multi frame
                                        log::debug!("ISOTP Start frame {data:02X?}");
                                        let mut data_tx: Vec<u8> = vec![];
                                        if rx_memory.receiving {
                                            data_tx.push(0x32);
                                        } else {
                                            data_tx.push(0x30);
                                            data_tx.push(cfg.block_size);
                                            data_tx.push(cfg.st_min);
                                            rx_memory.bs = cfg.block_size;
                                            rx_memory.add_start_frame(data);
                                        }
                                        if cfg.pad_frame {
                                            data_tx.resize(8, 0xCC);
                                        }
                                        // Send flow control
                                        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
                                        let f = CanFrame::new(
                                            default_tx_addr,
                                            &data_tx,
                                            cfg.can_use_ext_addr,
                                        );
                                        let _ = can_msg_sender_isotp.send(CanMessage::WriteFrames(
                                            0,
                                            vec![f],
                                            tx,
                                        ));
                                        let _ = rx.recv().unwrap();
                                    }
                                    0x20 => {
                                        // Continuation of multi frame
                                        log::debug!("ISOTP continue frame {data:02X?}");
                                        if IsoTpRxAction::SendFC
                                            == rx_memory.add_continuous_frame(data)
                                        {
                                            let mut data_tx: Vec<u8> = vec![];
                                            data_tx.push(0x30);
                                            data_tx.push(cfg.block_size);
                                            data_tx.push(cfg.st_min);
                                            rx_memory.bs = cfg.block_size;
                                            if cfg.pad_frame {
                                                data_tx.resize(8, 0xCC);
                                            }

                                            rx_memory.frames_received = 0; // Reset the counter

                                            let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
                                            let f = CanFrame::new(
                                                default_tx_addr,
                                                &data_tx,
                                                cfg.can_use_ext_addr,
                                            );
                                            let _ = can_msg_sender_isotp
                                                .send(CanMessage::WriteFrames(0, vec![f], tx));
                                            let _ = rx.recv().unwrap();
                                        }
                                    }
                                    0x30 => {
                                        // Flow control
                                        log::debug!("ISOTP Flow control {data:02X?}");
                                        tx_memory.on_flow_control(data);
                                    }
                                    _ => {
                                        log::error!("Invalid ISOTP CAN frame! {frame:?}");
                                    }
                                }
                            }
                            None => {
                                log::error!("ISOTP CAN frame too short! {frame:?}");
                            }
                        }
                    }
                }
                // Check for Rx status
                if bg_rx_receiver.is_some() {
                    if rx_memory.completed {
                        // Done!
                        let _ = bg_rx_receiver
                            .take()
                            .unwrap()
                            .send(Ok(rx_memory.data.clone()));
                        rx_memory.reset();
                    } else if rx_memory.last_rx_time.elapsed().as_millis() >= rx_timeout as u128 {
                        let _ = bg_rx_receiver
                            .take()
                            .unwrap()
                            .send(Err(ChannelError::ReadTimeout));
                        rx_memory.reset();
                    }
                }

                if tx_memory.transmitting {
                    if let Some(action_res) =
                        tx_memory.on_update(tx_timeout, isotp_settings.unwrap().pad_frame)
                    {
                        match action_res {
                            Ok(to_tx) => {
                                let cf = CanFrame::new(
                                    tx_memory.addr,
                                    &to_tx,
                                    isotp_settings.unwrap().can_use_ext_addr,
                                );
                                let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
                                let _ = can_msg_sender_isotp.send(CanMessage::WriteFrames(
                                    0,
                                    vec![cf],
                                    tx,
                                ));
                                if let Err(e) = rx.recv().unwrap() {
                                    if let Some(x) = bg_tx_receiver.take() {
                                        let _ = x.send(Err(e));
                                    }
                                    tx_memory.reset();
                                } else if tx_memory.completed {
                                    if let Some(x) = bg_tx_receiver.take() {
                                        let _ = x.send(Ok(()));
                                    }
                                    tx_memory.reset();
                                }
                            }
                            Err(e) => {
                                if let Some(x) = bg_tx_receiver.take() {
                                    let _ = x.send(Err(e));
                                }
                                tx_memory.reset();
                            }
                        }
                    }
                }
            }
        });

        // CAN channel dispatcher - Sends and receives CAN frames from raw interface
        std::thread::spawn(move || {
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
                    match msg {
                        CanMessage::Open(resp_sender) => {
                            let res = channel.open();
                            if res.is_ok() {
                                can_open.store(true, Ordering::Relaxed);
                            }
                            let _ = resp_sender.send(res);
                        }
                        CanMessage::Close(resp_sender) => {
                            let res = channel.close();
                            if res.is_ok() {
                                can_open.store(false, Ordering::Relaxed);
                            }
                            let _ = resp_sender.send(res);
                        }
                        CanMessage::Configure(baud, ext, resp_sender) => {
                            // If configurations are the same, then we can allow this
                            if let Some(current_cfg) = can_cfg {
                                if current_cfg == (baud, ext) {
                                    let _ = resp_sender.send(Ok(()));
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
                        }
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
                        }
                        CanMessage::WriteFrames(_timeout, f, resp_sender) => {
                            // TODO timeout
                            log::debug!("ISOTP Tx: [{:02X?}]", f[0].get_data());
                            let _ = resp_sender.send(channel.write_packets(f, 0));
                        }
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
                                let _ = can_to_isotp_rx_frame_tx.send(frame);
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
                            let _ = sender_read_res
                                .take()
                                .unwrap()
                                .send(Err(ChannelError::ReadTimeout));
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
            can_msg_sender,
            isotp_msg_sender,
        }
    }
}

impl PayloadChannel for SoftwareIsoTpChannel {
    fn open(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender
            .send(IsoTpMessage::Open(tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn close(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender
            .send(IsoTpMessage::Close(tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender
            .send(IsoTpMessage::SetIds(send, recv, tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        let (tx, rx) = mpsc::channel::<ChannelResult<Vec<u8>>>();
        self.isotp_msg_sender
            .send(IsoTpMessage::ReadBytes(timeout_ms, tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
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
        self.isotp_msg_sender
            .send(IsoTpMessage::WriteBytes(
                addr,
                ext_id,
                buffer.to_vec(),
                timeout_ms,
                tx,
            ))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender
            .send(IsoTpMessage::ClearRxBuffer(tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender
            .send(IsoTpMessage::ClearTxBuffer(tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }
}

impl IsoTPChannel for SoftwareIsoTpChannel {
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.isotp_msg_sender
            .send(IsoTpMessage::SetCfg(cfg, tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }
}

impl PacketChannel<CanFrame> for SoftwareIsoTpChannel {
    fn open(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender
            .send(CanMessage::Open(tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn close(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender
            .send(CanMessage::Close(tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, timeout_ms: u32) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender
            .send(CanMessage::WriteFrames(timeout_ms, packets, tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> ChannelResult<Vec<CanFrame>> {
        let (tx, rx) = mpsc::channel::<ChannelResult<Vec<CanFrame>>>();
        self.can_msg_sender
            .send(CanMessage::ReadFrames(max, timeout_ms, tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender
            .send(CanMessage::ClearRxBuffer(tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }
}

impl CanChannel for SoftwareIsoTpChannel {
    fn set_can_cfg(&mut self, baud: u32, use_extended: bool) -> ChannelResult<()> {
        let (tx, rx) = mpsc::channel::<ChannelResult<()>>();
        self.can_msg_sender
            .send(CanMessage::Configure(baud, use_extended, tx))
            .map_err(|e| ChannelError::Other(e.to_string()))?;
        rx.recv().unwrap()
    }
}
