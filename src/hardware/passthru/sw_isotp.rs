use std::borrow::BorrowMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use std::vec;

use crate::channel::{
    CanFrame, ChannelError, ChannelResult, IsoTPSettings, Packet, PacketChannel, PayloadChannel,
};

use crate::hardware::*;

use super::{PassthruCanChannel, PassthruDevice};

#[derive(Debug, Clone)]
enum ChannelMessage<T, X> {
    ClearRx,
    ClearTx,
    SendData { ext_id: Option<u8>, d: T },
    SetConfig(X),
    SetFilter(u32, u32), // Only for ISOTP
    Open,
    Close,
}

/// Passthru combination channel for software emulation of ISO-TP channel over CAN channel
///
/// ## Why?
/// According to the J2534 API, a CAN and ISO-TP cannot be opened at the same time, as they both require
/// physical access to the same hardware communication layer of the VCI.
///
/// To overcome this, we instead up open a dedicated CAN channel and run the ISO-TP communication via software.
/// This allows for both CAN and ISO-TP to coexist at the same time.
///
/// ## IMPORTANT NOTE
/// This mode is technically a violation of the J2534 API, whilst tested devices work fine with this
/// some cheap 'clone' J2534 adapters may struggle with the high throughput CAN Channel that is requied for this
/// to work.
#[allow(missing_debug_implementations)]
pub struct PtCombiChannel {
    can_rx_queue: mpsc::Receiver<CanFrame>,
    can_tx_queue: mpsc::Sender<ChannelMessage<CanFrame, (u32, bool)>>,
    can_tx_res_queue: mpsc::Receiver<ChannelResult<()>>,

    isotp_rx_queue: mpsc::Receiver<(u32, Vec<u8>)>,
    isotp_tx_queue: mpsc::Sender<ChannelMessage<(u32, Vec<u8>), IsoTPSettings>>,
    isotp_tx_res_queue: mpsc::Receiver<ChannelResult<()>>,

    can_mutex: Mutex<()>,
    isotp_mutex: Mutex<()>,
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

struct IsoTpPayload {
    pub data: Vec<u8>,
    pub curr_size: usize,
    pub max_size: usize,
    pub cts: bool,
    pub pci: u8,
    pub max_cpy_size: u8,
    pub ext_addr: bool,
    pub bs: u8,
    pub stmin: u8,
}

unsafe impl Sync for PtCombiChannel {}
unsafe impl Send for PtCombiChannel {}

fn parse_packet(cf: &CanFrame, iso_tp_ext: bool) -> (u32, &[u8]) {
    let data = cf.get_data();
    match iso_tp_ext {
        true => ((cf.get_address()) | data[0] as u32, &data[1..]),
        false => (cf.get_address(), data),
    }
}

impl PtCombiChannel {
    /// Creates a new combi channel using a given passthru device
    pub fn new(mut dev: PassthruDevice) -> HardwareResult<Self> {
        {
            log::debug!("SW Combi channel constructor called");
            if dev.can_channel.load(Ordering::Relaxed) || dev.isotp_channel.load(Ordering::Relaxed)
            {
                // Cannot proceed as dedicated CAN or ISO-TP dedicated channel is already open
                log::error!("CAN/ISO-TP dedicated channel already open");
                return Err(HardwareError::ConflictingChannel);
            }

            // We now own both channel types. This prevents further access to these channels
            dev.can_channel.store(true, Ordering::Relaxed);
            dev.isotp_channel.store(true, Ordering::Relaxed);
        }

        let (tx_can_send, rx_can_send) = mpsc::channel::<ChannelMessage<CanFrame, (u32, bool)>>();
        let (tx_can_send_res, rx_can_send_res) = mpsc::channel::<ChannelResult<()>>();
        let (tx_can_recv, rx_can_recv) = mpsc::channel::<CanFrame>();

        let (tx_isotp_send, rx_isotp_send) =
            mpsc::channel::<ChannelMessage<(u32, Vec<u8>), IsoTPSettings>>();
        let (tx_isotp_send_res, rx_isotp_send_res) = mpsc::channel::<ChannelResult<()>>();
        let (tx_isotp_recv, rx_isotp_recv) = mpsc::channel::<(u32, Vec<u8>)>();

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_t = is_running.clone();
        let handle = std::thread::spawn(move || {
            let mut channel: Option<PassthruCanChannel> = None;
            let mut iso_tp_cfg: Option<IsoTPSettings> = None;
            let can_cfg: Option<(u32, bool)> = None;
            let mut iso_tp_filter: Option<(u32, u32)> = None; // Tx, Rx
            let mut isotp_rx: Option<IsoTpPayload> = None;
            let mut isotp_tx: Option<IsoTpPayload> = None;
            let mut ext_address = false;
            let mut last_tx_time = Instant::now();
            let mut tx_frames_sent = 0u32;
            let mut rx_frames_received = 0u32;

            log::info!("SW ISO-TP thread running!");
            while is_running_t.load(Ordering::Relaxed) {
                if let Ok(can_req) = rx_can_send.try_recv() {
                    log::debug!("SW ISO-TP CAN Req: {can_req:?}");
                    tx_can_send_res.send(Ok(())).unwrap();
                }
                if let Ok(isotp_req) = rx_isotp_send.try_recv() {
                    let _send = match isotp_req {
                        ChannelMessage::ClearRx => {
                            isotp_rx = None;
                            tx_isotp_send_res.send(Ok(()))
                        }
                        ChannelMessage::ClearTx => {
                            isotp_tx = None;
                            tx_isotp_send_res.send(Ok(()))
                        } // Todo clear Tx buffer,
                        ChannelMessage::SendData {
                            ext_id: _,
                            d: (addr, data),
                        } => {
                            if iso_tp_cfg.is_none() || iso_tp_filter.is_none() {
                                tx_isotp_send_res.send(Err(ChannelError::ConfigurationError))
                            } else if channel.is_none() {
                                tx_isotp_send_res.send(Err(ChannelError::InterfaceNotOpen))
                            } else {
                                let cfg = iso_tp_cfg.unwrap();
                                let can = channel.as_mut().unwrap();
                                // Send
                                if (ext_address && data.len() < 6)
                                    || (!ext_address && data.len() < 7)
                                {
                                    let mut df: Vec<u8> = Vec::with_capacity(8);
                                    if ext_address {
                                        df.push(cfg.extended_addresses.unwrap().0);
                                    }
                                    df.push(data.len() as u8);
                                    df.extend_from_slice(&data);
                                    if cfg.pad_frame {
                                        df.resize(8, 0xCC);
                                    }
                                    log::debug!("Sending ISO-TP msg as 1 CAN frame {df:02X?}");
                                    let cf = CanFrame::new(addr, &df, cfg.can_use_ext_addr);
                                    tx_isotp_send_res.send(can.write_packets(vec![cf], 100))
                                } else {
                                    if isotp_tx.is_some() {
                                        tx_isotp_send_res.send(Err(ChannelError::BufferFull))
                                    } else if data.len() > 0xFFF {
                                        tx_isotp_send_res
                                            .send(Err(ChannelError::UnsupportedRequest))
                                    // Request too big
                                    } else {
                                        let mut df: Vec<u8> = Vec::with_capacity(8);
                                        if ext_address {
                                            df.push(cfg.extended_addresses.unwrap().0);
                                        }
                                        df.push((0x10 | ((data.len() as u32) >> 8) & 0x0F) as u8);
                                        df.push(data.len() as u8);
                                        let max_copy = if ext_address { 5 } else { 6 };
                                        df.extend_from_slice(&data[0..max_copy]);
                                        let cf = CanFrame::new(addr, &df, cfg.can_use_ext_addr);
                                        let res = can.write_packets(vec![cf], 100);
                                        if res.is_ok() {
                                            isotp_tx = Some(IsoTpPayload {
                                                data: data.clone(),
                                                curr_size: max_copy,
                                                max_size: data.len(),
                                                cts: false,
                                                pci: 0x21,
                                                max_cpy_size: max_copy as u8 + 1,
                                                ext_addr: ext_address,
                                                // These 2 are temp, they are overriden by the ECU when FC comes in
                                                bs: cfg.block_size,
                                                stmin: cfg.st_min,
                                            });
                                        }
                                        tx_isotp_send_res.send(Ok(()))
                                    }
                                }
                            }
                        }
                        ChannelMessage::SetConfig(cfg) => {
                            let mut res: ChannelResult<()> = Ok(());
                            if let Some(ccfg) = can_cfg {
                                // Compare against CAN config
                                if ccfg.0 != cfg.can_speed || ccfg.1 != cfg.can_use_ext_addr {
                                    // Mismatched config!
                                    res = Err(ChannelError::Other(
                                        "CAN and ISO-TP cfg mismatched for combi channel".into(),
                                    ));
                                }
                            }
                            if res.is_ok() {
                                ext_address = cfg.extended_addresses.is_some();
                                iso_tp_cfg = Some(cfg);
                            }
                            tx_isotp_send_res.send(res)
                        }
                        ChannelMessage::SetFilter(tx, rx) => {
                            iso_tp_filter = Some((tx, rx));
                            tx_isotp_send_res.send(Ok(()))
                        }
                        ChannelMessage::Open => {
                            let mut res: ChannelResult<()> = Ok(());
                            if iso_tp_cfg.is_none() {
                                res = Err(ChannelError::ConfigurationError)
                            }
                            if channel.is_none() {
                                // Try open channel
                                match dev.make_can_channel_raw() {
                                    Ok(c) => channel = Some(c),
                                    Err(e) => res = Err(e.into()),
                                };
                                if let Some(can_channel) = channel.borrow_mut() {
                                    let tp_cfg = iso_tp_cfg.unwrap();
                                    res = can_channel
                                        .set_can_cfg(tp_cfg.can_speed, tp_cfg.can_use_ext_addr);
                                    if res.is_ok() {
                                        res = can_channel.open();
                                    }
                                }
                            }
                            tx_isotp_send_res.send(res)
                        }
                        ChannelMessage::Close => {
                            iso_tp_cfg = None;
                            isotp_rx = None;
                            isotp_tx = None;
                            tx_isotp_send_res.send(Ok(()))
                        }
                    };
                }
                if let Some(c) = channel.borrow_mut() {
                    let incomming = c.read_packets(100, 0).unwrap_or_default();
                    if can_cfg.is_some() {
                        for p in &incomming {
                            tx_can_recv.send(*p).unwrap();
                        }
                    }
                    if let (Some(cfg), Some(filter)) = (iso_tp_cfg, iso_tp_filter) {
                        for packet in incomming {
                            if packet.get_address() == filter.1 {
                                if ext_address
                                    && packet.get_data()[1] != cfg.extended_addresses.unwrap().1
                                {
                                    continue;
                                }
                                // IsoTP is some so process the incomming frame!
                                // check PCI first (Quicker)
                                let pci_idx = if ext_address { 1 } else { 0 };
                                let pci_raw = *packet.get_data().get(pci_idx).unwrap_or(&0xFF);
                                let pci = pci_raw & 0xF0;
                                if pci == 0x00 || pci == 0x10 || pci == 0x20 || pci == 0x30 {
                                    let data = packet.get_data();
                                    log::debug!(
                                        "Incomming ISO-TP frame 0x{:04X?}: {:02X?}",
                                        filter.1,
                                        data
                                    );
                                    match pci {
                                        0x00 => {
                                            // Single frame
                                            tx_isotp_recv
                                                .send((
                                                    filter.1,
                                                    data[1 + pci_idx
                                                        ..1 + pci_idx + pci_raw as usize]
                                                        .to_vec(),
                                                ))
                                                .unwrap();
                                        }
                                        0x10 => {
                                            if isotp_rx.is_some() {
                                                log::warn!("ISOTP Rx overwriting old payload!");
                                            }
                                            let size = ((data[pci_idx] as usize & 0x0F) << 8)
                                                | data[1 + pci_idx] as usize;
                                            let mut data_rx = Vec::with_capacity(size);
                                            log::debug!("ISOTP Expecting data payload of {size} bytes, sending FC");
                                            data_rx.extend_from_slice(&data[pci_idx + 2..]);
                                            isotp_rx = Some(IsoTpPayload {
                                                data: data_rx,
                                                curr_size: 8 - 2 - pci_idx,
                                                max_size: size,
                                                cts: true,
                                                pci: 0x21,
                                                max_cpy_size: if ext_address { 6 } else { 7 },
                                                ext_addr: ext_address,
                                                bs: cfg.block_size,
                                                stmin: cfg.st_min,
                                            });
                                            // Send FC
                                            let mut df: Vec<u8> = Vec::with_capacity(8);
                                            if ext_address {
                                                df.push(cfg.extended_addresses.unwrap().0);
                                            }
                                            df.extend_from_slice(&[
                                                0x30,
                                                cfg.block_size,
                                                cfg.st_min,
                                            ]);
                                            if cfg.pad_frame {
                                                df.resize(8, 0xCC);
                                            }
                                            if let Err(e) = c.write_packets(
                                                vec![CanFrame::new(
                                                    filter.0,
                                                    &df,
                                                    cfg.can_use_ext_addr,
                                                )],
                                                100,
                                            ) {
                                                isotp_rx = None; // Could not send FC
                                                log::error!("Could not send FC to ECU: {e}");
                                            }
                                            rx_frames_received = 0;
                                        }
                                        0x20 => {
                                            if let Some(rx) = isotp_rx.borrow_mut() {
                                                let mut max_copy = rx.max_size - rx.data.len();
                                                if max_copy > rx.max_cpy_size as usize {
                                                    max_copy = rx.max_cpy_size as usize;
                                                }
                                                rx_frames_received += 1;
                                                rx.data.extend_from_slice(
                                                    &data[1 + pci_idx..1 + pci_idx + max_copy],
                                                );
                                                if rx.data.len() >= rx.max_size {
                                                    // Yay, done!
                                                    tx_isotp_recv
                                                        .send((filter.1, rx.data.clone()))
                                                        .unwrap();
                                                    isotp_rx = None;
                                                    continue;
                                                }
                                                // Not done, check if ECU requires a new FC msg
                                                if rx.bs > 0 && rx_frames_received >= rx.bs as u32 {
                                                    // Check for new fc required
                                                    // Send FC
                                                    let mut df: Vec<u8> = Vec::with_capacity(8);
                                                    if ext_address {
                                                        df.push(cfg.extended_addresses.unwrap().0);
                                                    }
                                                    df.extend_from_slice(&[
                                                        0x30,
                                                        cfg.block_size,
                                                        cfg.st_min,
                                                    ]);
                                                    if cfg.pad_frame {
                                                        df.resize(8, 0xCC);
                                                    }
                                                    if let Err(e) = c.write_packets(
                                                        vec![CanFrame::new(
                                                            filter.0,
                                                            &df,
                                                            cfg.can_use_ext_addr,
                                                        )],
                                                        100,
                                                    ) {
                                                        isotp_rx = None; // Could not send FC
                                                        log::error!(
                                                            "Could not send FC to ECU: {e}"
                                                        );
                                                    }
                                                    rx_frames_received = 0;
                                                    // Send FC
                                                }
                                            }
                                        }
                                        0x30 => {
                                            if pci_raw == 0x30 {
                                                if let Some(to_tx) = isotp_tx.as_mut() {
                                                    to_tx.cts = true;
                                                    to_tx.bs = data[1 + pci_idx];
                                                    to_tx.stmin = data[2 + pci_idx];
                                                    if to_tx.stmin > 127 {
                                                        to_tx.stmin = 1; // In microseconds, we don't count that fast, so use 1ms
                                                    }
                                                    last_tx_time = Instant::now();
                                                    tx_frames_sent = 0;
                                                }
                                            }
                                        }
                                        _ => {
                                            log::warn!("Cannot handle ISO-TP frame {data:02X?}");
                                        }
                                    }
                                }
                            }
                        }
                        let mut send_complete = false;
                        if let Some(tx_payload) = isotp_tx.borrow_mut() {
                            // Handle Tx data
                            let mut can_buffer = vec![];
                            for _ in 0..8 {
                                // 8 frames in a batch max - Makes Tx with 0bs faster
                                if tx_payload.cts
                                    && ((last_tx_time.elapsed().as_millis()
                                        >= tx_payload.stmin.into())
                                        || tx_payload.stmin == 0)
                                {
                                    let mut cf_payload = Vec::with_capacity(8);
                                    // Do send
                                    let max_copy = std::cmp::min(
                                        tx_payload.max_size - tx_payload.curr_size,
                                        tx_payload.max_cpy_size as usize,
                                    );

                                    if ext_address {
                                        cf_payload.push(cfg.extended_addresses.unwrap().0)
                                    }
                                    cf_payload.push(tx_payload.pci);
                                    cf_payload.extend_from_slice(
                                        &tx_payload.data
                                            [tx_payload.curr_size..tx_payload.curr_size + max_copy],
                                    );
                                    can_buffer.push(CanFrame::new(
                                        filter.0,
                                        &cf_payload,
                                        cfg.can_use_ext_addr,
                                    ));
                                    if cfg.pad_frame {
                                        cf_payload.resize(8, 0xCC);
                                    }

                                    if tx_payload.bs != 0 {
                                        tx_frames_sent += 1;
                                    }
                                    tx_payload.pci += 1;
                                    tx_payload.curr_size += max_copy;
                                    if tx_payload.pci == 0x30 {
                                        tx_payload.pci = 0x20;
                                    }

                                    // Await new FC
                                    last_tx_time = Instant::now();
                                    if tx_frames_sent as u8 >= tx_payload.bs && tx_payload.bs != 0 {
                                        tx_frames_sent = 0;
                                        tx_payload.cts = false;
                                        break;
                                    }
                                    if tx_payload.bs != 0 {
                                        break; // Delay
                                    }
                                    if tx_payload.curr_size >= tx_payload.max_size {
                                        send_complete = true;
                                        break;
                                    }
                                }
                            }
                            if !can_buffer.is_empty() && c.write_packets(can_buffer, 100).is_err() {
                                send_complete = true; // Destroy!
                            }
                        }
                        if send_complete {
                            isotp_tx = None;
                            log::debug!("ISO-TP Send completed!");
                        }
                    }
                }

                // Now decide how long to sleep for
                if iso_tp_cfg.is_none() {
                    std::thread::sleep(Duration::from_millis(10));
                } else {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            // Teardown
            log::info!("SW ISO-TP closed");
            if let Some(mut c) = channel.take() {
                c.close().unwrap();
                dev.can_channel.store(false, Ordering::Relaxed);
                dev.isotp_channel.store(false, Ordering::Relaxed);
            }
        });

        Ok(Self {
            can_rx_queue: rx_can_recv,
            can_tx_queue: tx_can_send,
            can_tx_res_queue: rx_can_send_res,

            isotp_rx_queue: rx_isotp_recv,
            isotp_tx_queue: tx_isotp_send,
            isotp_tx_res_queue: rx_isotp_send_res,
            can_mutex: Mutex::new(()),
            isotp_mutex: Mutex::new(()),
            running: is_running,
            handle: Some(handle),
        })
    }
}

impl CanChannel for PtCombiChannel {
    fn set_can_cfg(&mut self, baud: u32, use_extended: bool) -> ChannelResult<()> {
        log::debug!("CAN SetCANCfg called");
        let _guard = self.can_mutex.lock()?;
        while self.can_tx_res_queue.try_recv().is_ok() {}
        self.can_tx_queue
            .send(ChannelMessage::SetConfig((baud, use_extended)))?;
        // Wait for channels response
        self.can_tx_res_queue
            .recv_timeout(Duration::from_millis(100))?
    }
}

impl IsoTPChannel for PtCombiChannel {
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()> {
        log::debug!("ISO-TP SetIsoTpCfg called");
        let _guard = self.isotp_mutex.lock()?;
        while self.isotp_tx_res_queue.try_recv().is_ok() {}
        self.isotp_tx_queue.send(ChannelMessage::SetConfig(cfg))?;
        // Wait for channels response
        self.isotp_tx_res_queue
            .recv_timeout(Duration::from_millis(100))?
    }
}

impl PacketChannel<CanFrame> for PtCombiChannel {
    fn open(&mut self) -> ChannelResult<()> {
        log::debug!("CAN Open called");
        let _guard = self.can_mutex.lock()?;
        while self.can_tx_res_queue.try_recv().is_ok() {}
        self.can_tx_queue.send(ChannelMessage::Open)?;
        // Wait for channels response
        self.can_tx_res_queue
            .recv_timeout(Duration::from_millis(100))?
    }

    fn close(&mut self) -> ChannelResult<()> {
        log::debug!("CAN Close called");
        let _guard = self.can_mutex.lock()?;
        while self.can_tx_res_queue.try_recv().is_ok() {}
        self.can_tx_queue.send(ChannelMessage::Close)?;
        // Wait for channels response
        self.can_tx_res_queue
            .recv_timeout(Duration::from_millis(100))?
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, timeout_ms: u32) -> ChannelResult<()> {
        log::debug!("CAN WritePackets called");
        let _guard = self.can_mutex.lock()?;
        while self.can_tx_res_queue.try_recv().is_ok() {}
        for p in packets {
            self.can_tx_queue
                .send(ChannelMessage::SendData { ext_id: None, d: p })?;
            if timeout_ms != 0 {
                match self
                    .isotp_tx_res_queue
                    .recv_timeout(Duration::from_millis(timeout_ms as u64))
                {
                    Ok(m) => m?,
                    Err(e) => return Err(e.into()),
                }
            }
        }
        Ok(())
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> ChannelResult<Vec<CanFrame>> {
        log::debug!("CAN ReadPackets called");
        let timeout = std::cmp::max(1, timeout_ms);
        let mut res = vec![];
        let instant = Instant::now();
        while instant.elapsed().as_millis() <= timeout as u128 {
            if let Ok(c) = self.can_rx_queue.try_recv() {
                res.push(c)
            }
            if res.len() >= max {
                break;
            }
        }
        Ok(res)
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        log::debug!("CAN ClearRxBuffer called");
        while self.can_rx_queue.try_recv().is_ok() {}
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        log::debug!("CAN ClearTxBuffer called");
        Ok(self.can_tx_queue.send(ChannelMessage::ClearTx)?)
    }
}

impl PayloadChannel for PtCombiChannel {
    fn open(&mut self) -> ChannelResult<()> {
        log::debug!("ISO-TP Open called");
        let _guard = self.isotp_mutex.lock()?;
        while self.isotp_tx_res_queue.try_recv().is_ok() {}
        self.isotp_tx_queue.send(ChannelMessage::Open)?;
        // Wait for channels response
        self.isotp_tx_res_queue
            .recv_timeout(Duration::from_millis(100))?
    }

    fn close(&mut self) -> ChannelResult<()> {
        log::debug!("ISO-TP Close called");
        let _guard = self.isotp_mutex.lock()?;
        while self.isotp_tx_res_queue.try_recv().is_ok() {}
        self.isotp_tx_queue.send(ChannelMessage::Close)?;
        // Wait for channels response
        self.isotp_tx_res_queue
            .recv_timeout(Duration::from_millis(100))?
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        log::debug!("ISO-TP SetIDS called");
        let _guard = self.isotp_mutex.lock()?;
        while self.isotp_tx_res_queue.try_recv().is_ok() {}
        self.isotp_tx_queue
            .send(ChannelMessage::SetFilter(send, recv))?;
        // Wait for channels response
        self.isotp_tx_res_queue
            .recv_timeout(Duration::from_millis(100))?
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        log::debug!("ISO-TP ReadBytes called");
        let timeout = std::cmp::max(1, timeout_ms);
        let instant = Instant::now();
        while instant.elapsed().as_millis() <= timeout as u128 {
            if let Ok(c) = self.isotp_rx_queue.try_recv() {
                return Ok(c.1);
            }
        }
        Err(ChannelError::BufferEmpty)
    }

    fn write_bytes(
        &mut self,
        addr: u32,
        ext_id: Option<u8>,
        buffer: &[u8],
        timeout_ms: u32,
    ) -> ChannelResult<()> {
        log::debug!("ISO-TP WriteBytes called");
        let _guard = self.isotp_mutex.lock()?;
        while self.isotp_tx_res_queue.try_recv().is_ok() {}
        self.isotp_tx_queue.send(ChannelMessage::SendData {
            ext_id,
            d: (addr, buffer.to_vec()),
        })?;
        if timeout_ms == 0 {
            Ok(())
        } else {
            // Wait for channels response
            self.isotp_tx_res_queue
                .recv_timeout(Duration::from_millis(timeout_ms as u64))?
        }
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        log::debug!("ISO-TP ClearRxBuffer called");
        while self.isotp_rx_queue.try_recv().is_ok() {}
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        log::debug!("ISO-TP ClearTxBuffer called");
        Ok(self.isotp_tx_queue.send(ChannelMessage::ClearTx)?)
    }
}

impl Drop for PtCombiChannel {
    fn drop(&mut self) {
        log::debug!("Drop called");
        self.running.store(false, Ordering::Relaxed);
        self.handle.take().map(|x| x.join());
    }
}
