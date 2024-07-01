//! SocketCAN module

use std::{
    borrow::BorrowMut,
    io::ErrorKind,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use socketcan::Socket;

use socketcan::CanFrame as SocketCanCanFrame;
use socketcan_isotp::{
    ExtendedId, FlowControlOptions, Id, IsoTpBehaviour, IsoTpOptions, LinkLayerOptions, StandardId,
};

use crate::channel::{
    CanChannel, CanFrame, ChannelError, ChannelResult, IsoTPChannel, IsoTPSettings, PacketChannel,
    PayloadChannel,
};

use super::{Hardware, HardwareCapabilities, HardwareError, HardwareInfo, HardwareScanner};

const SOCKET_CAN_CAPABILITIES: HardwareCapabilities = HardwareCapabilities {
    iso_tp: true,
    can: true,
    ip: false,
    sae_j1850: false,
    kline: false,
    kline_kwp: false,
    sci: false,
};

/// SocketCAN device
#[derive(Clone, Debug)]
pub struct SocketCanDevice {
    info: HardwareInfo,
    canbus_active: Arc<AtomicBool>,
    isotp_active: Arc<AtomicBool>,
}

impl SocketCanDevice {
    pub(crate) fn new(if_name: String) -> Self {
        Self {
            info: HardwareInfo {
                name: if_name,
                vendor: None,
                capabilities: SOCKET_CAN_CAPABILITIES,
                device_fw_version: None,
                api_version: None,
                library_version: None,
                library_location: None,
            },
            canbus_active: Arc::new(AtomicBool::new(false)),
            isotp_active: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Hardware for SocketCanDevice {
    fn create_iso_tp_channel(&mut self) -> super::HardwareResult<Box<dyn IsoTPChannel>> {
        Ok(Box::new(SocketCanIsoTPChannel {
            device: self.clone(),
            channel: None,
            ids: (0, 0),
            cfg: IsoTPSettings::default(),
            cfg_complete: false,
        }))
    }

    fn create_can_channel(&mut self) -> super::HardwareResult<Box<dyn CanChannel>> {
        Ok(Box::new(SocketCanCanChannel {
            device: self.clone(),
            channel: None,
        }))
    }

    fn read_battery_voltage(&mut self) -> Option<f32> {
        None
    }

    fn read_ignition_voltage(&mut self) -> Option<f32> {
        None
    }

    fn get_info(&self) -> &HardwareInfo {
        &self.info
    }

    fn is_iso_tp_channel_open(&self) -> bool {
        self.isotp_active.load(Ordering::Relaxed)
    }

    fn is_can_channel_open(&self) -> bool {
        self.canbus_active.load(Ordering::Relaxed)
    }

    fn is_connected(&self) -> bool {
        PathBuf::from(format!("/sys/class/net/{}", self.info.name)).exists()
    }
}

#[derive(Debug)]
/// SocketCAN CAN channel
pub struct SocketCanCanChannel {
    device: SocketCanDevice,
    channel: Option<socketcan::CanSocket>,
}

impl SocketCanCanChannel {
    fn safe_with_iface<X, T: FnOnce(&mut socketcan::CanSocket) -> ChannelResult<X>>(
        &mut self,
        function: T,
    ) -> ChannelResult<X> {
        match self.channel.borrow_mut() {
            Some(channel) => function(channel),
            None => Err(ChannelError::InterfaceNotOpen),
        }
    }
}

impl PacketChannel<CanFrame> for SocketCanCanChannel {
    fn open(&mut self) -> ChannelResult<()> {
        if self.channel.is_some() {
            return Ok(()); // Already open!
        }
        let channel = socketcan::CanSocket::open(&self.device.info.name)?;

        channel.set_error_filter_drop_all()?;
        channel.set_filter_accept_all()?;

        self.channel = Some(channel);
        self.device.canbus_active.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn close(&mut self) -> ChannelResult<()> {
        if self.channel.is_none() {
            return Ok(());
        }
        self.channel = None;
        self.device.canbus_active.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, timeout_ms: u32) -> ChannelResult<()> {
        self.safe_with_iface(|iface| {
            if timeout_ms == 0 {
                iface.set_nonblocking(true)?;
            } else {
                iface.set_nonblocking(false)?;
                iface.set_write_timeout(Duration::from_millis(timeout_ms as u64))?;
            }
            let mut cf: SocketCanCanFrame;
            for p in packets {
                cf = SocketCanCanFrame::Data(p.into());
                iface.write_frame(&cf)?;
            }
            Ok(())
        })
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> ChannelResult<Vec<CanFrame>> {
        self.safe_with_iface(|iface| {
            let mut result: Vec<CanFrame> = Vec::new();
            if timeout_ms == 0 {
                iface.set_nonblocking(true)?;
                while let Ok(f) = iface.read_frame() {
                    if let SocketCanCanFrame::Data(d) = f {
                        result.push(d.into())
                    }
                    if result.len() == max {
                        break;
                    }
                }
                if result.len() == 0 {
                    Err(ChannelError::BufferEmpty)
                } else {
                    Ok(result)
                }
            } else {
                iface.set_nonblocking(false)?;
                iface.set_read_timeout(Duration::from_millis(timeout_ms as u64))?;
                let start = Instant::now();
                while start.elapsed().as_millis() <= timeout_ms as u128 {
                    let f = iface.read_frame()?;
                    if let SocketCanCanFrame::Data(d) = f {
                        result.push(d.into())
                    }
                    if result.len() == max {
                        break;
                    }
                }
                if result.len() == 0 {
                    Err(ChannelError::BufferEmpty)
                } else {
                    Ok(result)
                }
            }
        })
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        while self.read_packets(1, 0).is_ok() {}
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }
}

impl CanChannel for SocketCanCanChannel {
    /// SocketCAN ignores this function as the channel is pre-configured
    /// by the OS' kernel.
    fn set_can_cfg(&mut self, _baud: u32, _use_extended: bool) -> ChannelResult<()> {
        Ok(())
    }
}

impl Drop for SocketCanCanChannel {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.close();
    }
}

/// SocketCAN CAN channel
pub struct SocketCanIsoTPChannel {
    device: SocketCanDevice,
    channel: Option<socketcan_isotp::IsoTpSocket>,
    /// Tx ID, Rx ID
    ids: (u32, u32),
    cfg: IsoTPSettings,
    cfg_complete: bool,
}

impl SocketCanIsoTPChannel {
    fn safe_with_iface<X, T: FnOnce(&mut socketcan_isotp::IsoTpSocket) -> ChannelResult<X>>(
        &mut self,
        function: T,
    ) -> ChannelResult<X> {
        match self.channel.as_mut() {
            Some(channel) => function(channel),
            None => Err(ChannelError::InterfaceNotOpen),
        }
    }
}

impl std::fmt::Debug for SocketCanIsoTPChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocketCanIsoTPChannel")
            .field("device", &self.device)
            .finish()
    }
}

impl PayloadChannel for SocketCanIsoTPChannel {
    fn open(&mut self) -> ChannelResult<()> {
        if self.channel.is_some() {
            // Already open
            return Ok(());
        }
        let mut flags: IsoTpBehaviour = IsoTpBehaviour::empty();

        if self.cfg.extended_addresses.is_some() {
            flags |= IsoTpBehaviour::CAN_ISOTP_EXTEND_ADDR
        }
        if self.cfg.pad_frame {
            flags =
                flags | IsoTpBehaviour::CAN_ISOTP_TX_PADDING | IsoTpBehaviour::CAN_ISOTP_TX_PADDING
        }

        let mut ext_address: u8 = 0;
        let mut rx_ext_address: u8 = 0;
        if let Some((tx, rx)) = self.cfg.extended_addresses {
            ext_address = tx;
            rx_ext_address = rx;
        }

        let opts: IsoTpOptions = IsoTpOptions::new(
            flags,
            Duration::from_millis(0),
            ext_address,
            0xCC,
            0xCC,
            rx_ext_address,
        )
        .unwrap();

        let link_opts: LinkLayerOptions = LinkLayerOptions::default();

        let (tx_id, rx_id) = match self.cfg.can_use_ext_addr {
            true => (
                Id::Extended(unsafe { ExtendedId::new_unchecked(self.ids.0) }),
                Id::Extended(unsafe { ExtendedId::new_unchecked(self.ids.1) }),
            ),
            false => (
                Id::Standard(unsafe { StandardId::new_unchecked(self.ids.0 as u16) }),
                Id::Standard(unsafe { StandardId::new_unchecked(self.ids.1 as u16) }),
            ),
        };

        let fc_opts = FlowControlOptions::new(self.cfg.block_size, self.cfg.st_min, 0);

        let socket = socketcan_isotp::IsoTpSocket::open_with_opts(
            &self.device.info.name,
            rx_id,
            tx_id,
            Some(opts),
            Some(fc_opts),
            Some(link_opts),
        )?;
        socket.set_nonblocking(true)?;
        self.device.isotp_active.store(true, Ordering::Relaxed);
        self.channel = Some(socket);
        Ok(())
    }

    fn close(&mut self) -> ChannelResult<()> {
        if self.channel.is_none() {
            // Already shut
            return Ok(());
        }
        self.channel = None; // Closes channel
        self.device.isotp_active.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        self.ids = (send, recv);
        Ok(())
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        let start = Instant::now();
        let timeout = std::cmp::max(1, timeout_ms);
        self.safe_with_iface(|socket| {
            while start.elapsed().as_millis() <= timeout as u128 {
                if let Ok(data) = socket.read() {
                    return Ok(data.to_vec());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            // Timeout
            if timeout_ms == 0 {
                Err(ChannelError::BufferEmpty)
            } else {
                Err(ChannelError::ReadTimeout)
            }
        })
    }

    /// Writes bytes to socketcan socket.
    ///
    /// NOTE: Due to how ISO-TP channeling on SocketCAN works, there is a limitation when sending on a different address
    /// to what was defined in [Self::set_iso_tp_cfg]. It should work for most alternate address messages (EG: Global tester present),
    /// but longer messages will fail.
    ///
    /// If `buffer` is less than 7 bytes (With Standard ISO-TP addressing), or less than 6 bytes (With Extended ISO-TP addressing),
    /// this function will attempt to open a parallel socketCAN channel in order to send an ISO-TP single frame request on the alternate requested
    /// address.
    ///
    /// If `buffer` is more than 7 bytes and you request on an alternate address, then this function will fail with [ChannelError::UnsupportedRequest]
    fn write_bytes(
        &mut self,
        addr: u32,
        ext_id: Option<u8>,
        buffer: &[u8],
        timeout_ms: u32,
    ) -> ChannelResult<()> {
        // if ext_id is specified, this can override the cfg config extended addresses
        let mut ext_addresses = self.cfg.extended_addresses;
        if let Some(id) = ext_id {
            log::warn!("extended_addresses was specified byt ext_id also was. ext_id overriding");
            ext_addresses = Some((id, 0));
        }

        // Work around for issue #1
        // If the buffer is less than 7/6 bytes, we can send it as 1 frame (Usually for global tester present msg)
        // If this is the case, we can simply open a socketCAN channel to send that frame in parallel to the ISO-TP channel already open!
        if addr != self.ids.0 {
            if (buffer.len() <= 7 && ext_addresses.is_none())
                || (buffer.len() <= 6 && ext_addresses.is_some())
            {
                let mut data = Vec::new();
                let can_id = addr;
                if let Some((tx, _rx)) = ext_addresses {
                    data.push(tx);
                    data.push(buffer.len() as u8);
                } else {
                    // Ext ISO-TP addr
                    data.push(buffer.len() as u8);
                };
                data.extend_from_slice(buffer); // Push Tx Data

                if self.cfg.pad_frame {
                    // Pad to 8 bytes
                    data.resize(8, 0x00);
                }

                let can_frame = CanFrame::new(can_id, &data, self.cfg.can_use_ext_addr);

                let mut channel = self.device.create_can_channel()?;
                channel.open()?;
                channel.write_packets(vec![can_frame], timeout_ms)?;
                drop(channel);
                Ok(())
            } else {
                Err(ChannelError::UnsupportedRequest)
            }
        } else if ext_id.is_some() && self.cfg.extended_addresses.is_none() {
            // Create a temporary channel to send this one packet!
            let mut c_2 = self.device.create_iso_tp_channel()?;
            let cfg = IsoTPSettings {
                extended_addresses: Some((ext_id.unwrap(), 0)),
                ..self.cfg
            };
            c_2.set_iso_tp_cfg(cfg)?;
            c_2.set_ids(self.ids.0, self.ids.1)?;
            c_2.open()?;
            c_2.write_bytes(addr, None, buffer, timeout_ms)?;
            drop(c_2);
            Ok(())
        } else {
            // Easy
            self.safe_with_iface(|socket| {
                socket.write(buffer)?;
                Ok(())
            })
        }
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        self.safe_with_iface(|socket| {
            while socket.read().is_ok() {}
            Ok(())
        })
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }
}

impl IsoTPChannel for SocketCanIsoTPChannel {
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()> {
        self.cfg = cfg;
        // Try to set the baudrate
        self.cfg_complete = true;
        Ok(())
    }
}

impl Drop for SocketCanIsoTPChannel {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Debug)]
/// Socket CAN device scanner
pub struct SocketCanScanner {
    devices: Vec<HardwareInfo>,
}

impl Default for SocketCanScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl SocketCanScanner {
    /// Creates a new SocketCAN device scanner
    pub fn new() -> Self {
        match std::fs::read_dir("/sys/class/net/") {
            Ok(paths) => Self {
                devices: paths
                    .into_iter()
                    .map(|x| x.map(|e| e.path()))
                    .filter_map(|x| x.ok())
                    .map(|f| f.to_str().unwrap().to_string())
                    .map(|f| f.split('/').map(|s| s.to_string()).collect::<Vec<String>>())
                    .filter(|f| f.last().unwrap().contains("can"))
                    .map(|path| HardwareInfo {
                        name: path[path.len() - 1].clone(),
                        vendor: None,
                        capabilities: SOCKET_CAN_CAPABILITIES,
                        device_fw_version: None,
                        api_version: None,
                        library_version: None,
                        library_location: None,
                    })
                    .collect(),
            },
            Err(_) => Self {
                devices: Vec::new(),
            },
        }
    }
}

impl HardwareScanner<SocketCanDevice> for SocketCanScanner {
    fn list_devices(&self) -> Vec<HardwareInfo> {
        self.devices.clone()
    }

    fn open_device_by_index(&self, idx: usize) -> super::HardwareResult<SocketCanDevice> {
        match self.devices.get(idx) {
            Some(hw) => Ok(SocketCanDevice::new(hw.name.clone())),
            None => Err(HardwareError::DeviceNotFound),
        }
    }

    fn open_device_by_name(&self, name: &str) -> super::HardwareResult<SocketCanDevice> {
        match self.devices.iter().find(|x| x.name == name) {
            Some(hw) => Ok(SocketCanDevice::new(hw.name.clone())),
            None => Err(HardwareError::DeviceNotFound),
        }
    }
}

impl From<socketcan_isotp::Error> for ChannelError {
    fn from(e: socketcan_isotp::Error) -> Self {
        Self::HardwareError(HardwareError::APIError {
            code: 99,
            desc: e.to_string(),
        })
    }
}

impl From<std::io::Error> for ChannelError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == ErrorKind::WouldBlock {
            Self::BufferEmpty
        } else {
            Self::IOError(Arc::new(e))
        }
    }
}
