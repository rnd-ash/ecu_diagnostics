use std::sync::mpsc;
use std::time::Instant;

use crate::channel::{PacketChannel, CanFrame, PayloadChannel, IsoTPSettings, ChannelError};

use crate::hardware::*;

use super::{PassthruDevice, PassthruCanChannel};

#[derive(Debug, Clone)]
enum ChannelMessage<T> {
    ClearRx,
    ClearTx,
    SendData(T)
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
    dev: Arc<Mutex<PassthruDevice>>,
    iso_tp_cfg: Option<IsoTPSettings>,
    can_cfg: Option<(u32, bool)>, // baud, extended
    channel: PassthruCanChannel,

    can_rx_queue: mpsc::Receiver<CanFrame>,
    can_tx_queue: mpsc::Sender<ChannelMessage<CanFrame>>,

    isotp_rx_queue: mpsc::Receiver<(u32, Vec<u8>)>,
    isotp_tx_queue: mpsc::Sender<ChannelMessage<(u32, Vec<u8>)>>,

}

unsafe impl Sync for PtCombiChannel{}
unsafe impl Send for PtCombiChannel{}

impl PtCombiChannel {
    /// Creates a new combi channel using a given passthru device
    pub fn new(dev: Arc<Mutex<PassthruDevice>>) -> HardwareResult<Self> {
        {
            let mut this = dev.lock().unwrap();
            if this.can_channel || this.isotp_channel {
                // Cannot proceed as dedicated CAN or ISO-TP dedicated channel is already open
                return Err(HardwareError::ConflictingChannel)
            }
            
            // We now own both channel types. This prevents further access to these channels
            this.can_channel = true;
            this.isotp_channel = true;
        }

        let c = PassthruDevice::make_can_channel_raw(dev.clone())?;

        let (tx_can_send, rx_can_send) = mpsc::channel::<ChannelMessage<CanFrame>>();
        let (tx_can_recv, rx_can_recv) = mpsc::channel::<CanFrame>();

        let (tx_isotp_send, rx_isotp_send) = mpsc::channel::<ChannelMessage<(u32, Vec<u8>)>>();
        let (tx_isotp_recv, rx_isotp_recv) = mpsc::channel::<(u32, Vec<u8>)>();




        Ok(Self {
            dev,
            iso_tp_cfg: None,
            can_cfg: None,
            channel: c,
            can_rx_queue: rx_can_recv,
            can_tx_queue: tx_can_send,

            isotp_rx_queue: rx_isotp_recv,
            isotp_tx_queue: tx_isotp_send
        })
    }
}


impl CanChannel for PtCombiChannel {
    fn set_can_cfg(&mut self, baud: u32, use_extended: bool) -> crate::channel::ChannelResult<()> {
        if let Some(icfg) = self.iso_tp_cfg {
            if icfg.can_use_ext_addr != use_extended || icfg.can_speed != baud {
                // Mismatched settings!
                return Err(ChannelError::HardwareError(HardwareError::APIError { 
                    code: 99, 
                    desc: "CAN channel settings incompatible with open already active ISOTP settings".into()
                }))
            }
        }
        self.can_cfg = Some((baud, use_extended));
        Ok(())
    }
}

impl IsoTPChannel for PtCombiChannel {
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> crate::channel::ChannelResult<()> {
        if let Some(ccfg) = self.can_cfg {
            if ccfg.1 != cfg.can_use_ext_addr || ccfg.0 != cfg.can_speed {
                // Mismatched settings!
                return Err(ChannelError::HardwareError(HardwareError::APIError { 
                    code: 99, 
                    desc: "ISOTP channel settings incompatible with open already active CAN settings".into()
                }))
            }
        }
        self.iso_tp_cfg = Some(cfg);
        Ok(())
    }
}

impl PacketChannel<CanFrame> for PtCombiChannel {
    fn open(&mut self) -> crate::channel::ChannelResult<()> {
        if self.can_cfg.is_none() {
            return Err(ChannelError::ConfigurationError)
        }
        // Can channel already open! So OK
        Ok(())
    }

    fn close(&mut self) -> crate::channel::ChannelResult<()> {
        self.can_cfg = None;
        Ok(())
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, _timeout_ms: u32) -> crate::channel::ChannelResult<()> {
        for p in packets {
            self.can_tx_queue.send(ChannelMessage::SendData(p))?;
        }
        Ok(())
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> crate::channel::ChannelResult<Vec<CanFrame>> {
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

    fn clear_rx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        while self.can_rx_queue.recv().is_ok(){}
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        Ok(self.can_tx_queue.send(ChannelMessage::ClearTx)?)
    }
}

impl PayloadChannel for PtCombiChannel {
    fn open(&mut self) -> crate::channel::ChannelResult<()> {
        if self.iso_tp_cfg.is_none() {
            return Err(ChannelError::ConfigurationError)
        }
        Ok(())
    }

    fn close(&mut self) -> crate::channel::ChannelResult<()> {
        self.iso_tp_cfg = None;
        self.dev.lock().unwrap().isotp_channel = false;
        Ok(())
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> crate::channel::ChannelResult<()> {
        todo!()
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> crate::channel::ChannelResult<Vec<u8>> {
        let timeout = std::cmp::max(1, timeout_ms);
        let instant = Instant::now();
        while instant.elapsed().as_millis() <= timeout as u128 {
            if let Ok(c) = self.isotp_rx_queue.try_recv() {
                return Ok(c.1);
            }
        }
        Err(ChannelError::BufferEmpty)
    }

    fn write_bytes(&mut self, addr: u32, buffer: &[u8], _timeout_ms: u32) -> crate::channel::ChannelResult<()> {
        self.isotp_tx_queue.send(ChannelMessage::SendData((addr, buffer.to_vec())))?;
        Ok(())
    }

    fn clear_rx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        while self.isotp_rx_queue.recv().is_ok(){}
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        Ok(self.isotp_tx_queue.send(ChannelMessage::ClearTx)?)
    }
}

impl Drop for PtCombiChannel {
    fn drop(&mut self) {
        self.channel.close();
        {
            let mut this = self.dev.lock().unwrap();
            this.can_channel = false;
            this.isotp_channel = false;
        }
    }
}