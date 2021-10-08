//! SocketCAN module

use std::{sync::{Arc, Mutex}, time::Instant};

use socketcan_isotp::{IsoTpBehaviour, IsoTpOptions, LinkLayerOptions};

use crate::channel::{CanChannel, CanFrame, ChannelError, ChannelResult, IsoTPChannel, IsoTPSettings, Packet, PacketChannel, PayloadChannel};

use super::{Hardware, HardwareCapabilities, HardwareError, HardwareInfo, HardwareScanner};

const SOCKET_CAN_CAPABILITIES: HardwareCapabilities = HardwareCapabilities {
    iso_tp: true,
    can: true,
    ip: false,
    sae_j1850: false,
    kline: false,
    kline_kwp: false,
    sci: false
};

/// SocketCAN device
#[derive(Debug)]
pub struct SocketCanDevice{
    if_name: String,
    canbus_active: bool,
    isotp_active: bool
}

impl SocketCanDevice {
    pub (crate) fn new(if_name: String) -> Self {
        Self {
            if_name,
            canbus_active: false,
            isotp_active: false
        }
    }
}

impl Hardware for SocketCanDevice {
    fn create_iso_tp_channel(this: std::sync::Arc<std::sync::Mutex<Self>>) -> super::HardwareResult<Box<dyn crate::channel::IsoTPChannel>> {
        let device = this.lock()?;
        if device.canbus_active {
            return Err(HardwareError::ConflictingChannel)
        }
        Ok(Box::new(SocketCanIsoTPChannel {
            device: this.clone(),
            channel: None,
            ids: (0, 0),
            cfg: IsoTPSettings::default(),
            cfg_complete: false
        }))
    }

    fn create_can_channel(this: std::sync::Arc<std::sync::Mutex<Self>>) -> super::HardwareResult<Box<dyn crate::channel::CanChannel>> {
        let device = this.lock()?;
        if device.isotp_active {
            return Err(HardwareError::ConflictingChannel)
        }
        Ok(Box::new(SocketCanCanChannel {
            device: this.clone(),
            channel: None,
        }))
    }

    fn read_battery_voltage(&mut self) -> Option<f32> {
        None
    }

    fn get_capabilities(&self) -> &HardwareCapabilities {
        &SOCKET_CAN_CAPABILITIES
    }
}


#[derive(Debug)]
/// SocketCAN CAN channel
pub struct SocketCanCanChannel {
    device: Arc<Mutex<SocketCanDevice>>,
    channel: Option<socketcan::CANSocket>
}

impl SocketCanCanChannel {
    fn safe_with_iface<X, T: FnOnce(&socketcan::CANSocket) -> ChannelResult<X>>(&mut self, function: T) -> ChannelResult<X> {
        match self.channel {
            Some(ref channel) => {
                function(channel)
            },
            None => Err(ChannelError::InterfaceNotOpen)
        }
    }
}

impl PacketChannel<CanFrame> for SocketCanCanChannel {
    fn open(&mut self) -> crate::channel::ChannelResult<()> {
        if self.channel.is_some() {
            return Ok(()); // Already open!
        }
        let mut device = self.device.lock()?;
        let channel = socketcan::CANSocket::open(&device.if_name)?;
        channel.filter_accept_all()?;
        self.channel = Some(channel);
        device.canbus_active = true;
        Ok(())
    }

    fn close(&mut self) -> crate::channel::ChannelResult<()> {
        if self.channel.is_none() {
            return Ok(())
        }
        let mut device = self.device.lock()?;
        self.channel = None;
        device.canbus_active = false;
        Ok(())
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, timeout_ms: u32) -> crate::channel::ChannelResult<()> {
        self.safe_with_iface(|iface| {
            iface.set_write_timeout(std::time::Duration::from_millis(timeout_ms as u64))?;
            let mut cf: socketcan::CANFrame;
            for p in &packets {
                cf = socketcan::CANFrame::new(p.get_address(), p.get_data(), false, false).unwrap();
                iface.write_frame(&cf)?;
            }
            Ok(())
        })
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> crate::channel::ChannelResult<Vec<CanFrame>> {
        let timeout = std::cmp::max(1, timeout_ms) as u128;
        let mut result: Vec<CanFrame> = Vec::with_capacity(max);
        self.safe_with_iface(|iface| {
            let start = Instant::now();
            let mut read: socketcan::CANFrame;
            while start.elapsed().as_millis() <= timeout {
                read = iface.read_frame()?;
                result.push(CanFrame::new(read.id(), read.data(), read.is_extended()));
                // Read complete
                if result.len() == max {
                    return Ok(())
                }
            }
            Ok(())
        })?;
        result.shrink_to_fit(); // Deallocate unneeded memory
        Ok(result)
    }

    fn clear_rx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        Ok(())
    }
}

impl CanChannel for SocketCanCanChannel {
    /// SocketCAN ignores this function as the channel is pre-configured
    /// by the OS' kernel.
    fn set_can_cfg(&mut self, _baud: u32, _use_extended: bool) -> crate::channel::ChannelResult<()> {
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
    device: Arc<Mutex<SocketCanDevice>>,
    channel: Option<socketcan_isotp::IsoTpSocket>,
    ids: (u32, u32),
    cfg: IsoTPSettings,
    cfg_complete: bool,
}

impl SocketCanIsoTPChannel {
    fn safe_with_iface<X, T: FnOnce(&mut socketcan_isotp::IsoTpSocket) -> ChannelResult<X>>(&mut self, function: T) -> ChannelResult<X> {
        match self.channel.as_mut() {
            Some(channel) => {
                function(channel)
            },
            None => Err(ChannelError::InterfaceNotOpen)
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
        if self.channel.is_some() { // Already open
            return Ok(())
        }
        let mut device = self.device.lock()?;
        let mut flags: IsoTpBehaviour = IsoTpBehaviour::empty();

        if self.cfg.extended_addressing {
            flags |= IsoTpBehaviour::CAN_ISOTP_EXTEND_ADDR
        }
        if self.cfg.pad_frame {
            flags = flags | IsoTpBehaviour::CAN_ISOTP_TX_PADDING | IsoTpBehaviour::CAN_ISOTP_TX_PADDING
        }

        let mut ext_address: u8 = 0;
        let mut rx_ext_address: u8 = 0;
        if self.cfg.extended_addressing {
            ext_address = self.ids.0 as u8;
            rx_ext_address = self.ids.1 as u8;
        }

        let opts: IsoTpOptions = IsoTpOptions::new(flags, std::time::Duration::from_millis(0), ext_address, 0x00, 0x00, rx_ext_address).unwrap();
        let link_opts: LinkLayerOptions = LinkLayerOptions::default();
        let socket = socketcan_isotp::IsoTpSocket::open_with_opts(
            &device.if_name, 
            self.ids.1, 
            self.ids.0,
            Some(opts),
            None,
            Some(link_opts)
        )?;
        socket.set_nonblocking(true)?;
        device.canbus_active = true;
        self.channel = Some(socket);
        Ok(())
    }

    fn close(&mut self) -> ChannelResult<()> {
        let mut device = self.device.lock()?;
        if self.channel.is_none() { // Already shut
            return Ok(())
        }
        self.channel = None; // Closes channel
        device.canbus_active = false;
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
                    return Ok(data.to_vec())
                }
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
    /// NOTE: There is currently a bug where [addr] is ignored! See [this issue][1]
    /// 
    /// [1] https://github.com/rnd-ash/ecu_diagnostics/issues/1
    fn write_bytes(&mut self, _addr: u32, buffer: &[u8], _timeout_ms: u32) -> ChannelResult<()> {
        self.safe_with_iface(|socket| {
            socket.write(buffer)?;
            Ok(())
        })
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }
}

impl IsoTPChannel for SocketCanIsoTPChannel {
    fn set_iso_tp_cfg(&mut self, cfg: crate::channel::IsoTPSettings) -> ChannelResult<()> {
        self.cfg = cfg;
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
pub struct SocketCanScanner{
    devices: Vec<HardwareInfo>
}

impl std::default::Default for SocketCanScanner {
    fn default() -> Self {
        Self::new()
    }
}


impl SocketCanScanner {
    /// Creates a new SocketCAN device scanner
    pub fn new() -> Self {
        match std::fs::read_dir("/sys/class/net/") {
            Ok(paths) => {
                Self {
                    devices: paths.into_iter()
                    .map(|x| x.map(|e| e.path()))
                    .filter_map(|x| x.ok())
                    .map(|f| f.to_str().unwrap().to_string())
                    .map(|f| f.split('/').map(|s| s.to_string()).collect::<Vec<String>>())
                    .filter(|f| f.last().unwrap().contains("can"))
                    .map(|path| HardwareInfo {
                        name: path[path.len()-1].clone(),
                        vendor: "N/A".into(),
                        capabilities: SOCKET_CAN_CAPABILITIES,
                    })
                    .collect()   
                }
            }
            Err(_) => Self {
                devices: Vec::new()
            }
        }
    }
}

impl HardwareScanner<SocketCanDevice> for SocketCanScanner {
    fn list_devices(&self) -> Vec<HardwareInfo> {
        self.devices.clone()
    }

    fn open_device_by_index(&self, idx: usize) -> super::HardwareResult<std::sync::Arc<std::sync::Mutex<SocketCanDevice>>> {
        match self.devices.get(idx) {
            Some(hw) => Ok(Arc::new(Mutex::new(SocketCanDevice::new(hw.name.clone())))),
            None => Err(HardwareError::DeviceNotFound)
        }
    }

    fn open_device_by_name(&self, name: &str) -> super::HardwareResult<std::sync::Arc<std::sync::Mutex<SocketCanDevice>>> {
        match self.devices.iter().find(|x| x.name == name) {
            Some(hw) => Ok(Arc::new(Mutex::new(SocketCanDevice::new(hw.name.clone())))),
            None => Err(HardwareError::DeviceNotFound)
        }
    }
}


impl From<socketcan::CANSocketOpenError> for ChannelError {
    fn from(e: socketcan::CANSocketOpenError) -> Self {
        Self::HardwareError(
            HardwareError::APIError {
                code: 99,
                desc: e.to_string(),
            }
        )
    }
}

impl From<socketcan_isotp::Error> for ChannelError {
    fn from(e: socketcan_isotp::Error) -> Self {
        Self::HardwareError(
            HardwareError::APIError {
                code: 99,
                desc: e.to_string(),
            }
        )
    }
}

impl From<std::io::Error> for ChannelError {
    fn from(e: std::io::Error) -> Self {
        Self::IOError(e)
    }
}