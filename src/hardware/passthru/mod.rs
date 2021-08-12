//! The passthru API (Also known as SAE J2534) is an adapter protocol used by some OBD2 adapters.
//!
//! This module provides support for V04.04 of the API, including experimental support for OSX and Linux, used by
//! [Macchina-J2534][1]
//!
//! [1]: http://github.com/rnd-ash/macchina-J2534
//!
//! The API supports the following communication protocols:
//! * ISO9141 
//! * ISO15475
//! * ISO14230-4
//! * J1850 PWM
//! * J1850 VPW
//! * SCI
//! * CAN
//!
//! however it should be noted that adapters might only support a range of these protocols. So
//! querying the [super::HardwareCapabilities] matrix should be used to determine which protocols
//! are supported

use std::{borrow::{Borrow, BorrowMut}, cell::RefCell, convert::TryInto, ffi::c_void, sync::{Arc, Mutex}};

use j2534_rust::{ConnectFlags, IoctlID, Loggable, PASSTHRU_MSG, PassthruError, TxFlag};

use crate::channel::{CanChannel, CanFrame, ChannelError, IsoTPChannel, Packet, PacketChannel};

use self::lib_funcs::PassthruDrv;

use super::{HardwareCapabilities, HardwareError, HardwareInfo, HardwareResult};

mod lib_funcs;

/// Passthru API device scanner
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PassthruScanner{
    devices: Vec<PassthruInfo>
}

impl PassthruScanner {
    #[cfg(unix)]
    /// Creates a passthru scanner
    pub fn new() -> Self {
        match std::fs::read_dir(shellexpand::tilde("~/.passthru").to_string()) {
            Ok(list) => {
                Self {
                    devices: list
                        .into_iter()
                        // Remove files that cannot be read
                        .filter_map(|p| p.ok())
                        // Filter any files that are not json files
                        .filter(|p| p.file_name().to_str().unwrap().ends_with(".json"))
                        // Attempt to read a PassthruDevice from each json file found
                        .map(|p| PassthruInfo::new(&p.path()))
                        // Keep Oks that were found, any entries that ended with errors are discarded
                        .filter_map(|s| s.ok())
                        // Convert result into vector
                        .collect()
                }
            }
            Err(_) => Self {
                devices: Vec::new()
            },
        }
    }

    #[cfg(windows)]
    pub fn new() -> Self {
        todo!()
    }
}

impl super::HardwareScanner<PassthruDevice> for PassthruScanner {
    fn list_devices(&self) -> Vec<super::HardwareInfo> {
        self.devices.iter().map(|x| x.into()).collect()
    }

    fn open_device_by_index(&self, idx: usize) -> super::HardwareResult<PassthruDevice> {
        match self.devices.get(idx) {
            Some(info) => PassthruDevice::open_device(info),
            None => Err(HardwareError::DeviceNotFound),
        }
    }

    fn open_device_by_name(&self, name: &str) -> super::HardwareResult<PassthruDevice> {
        match self.devices.iter().find(|s| s.name == name) {
            Some(info) => PassthruDevice::open_device(info),
            None => Err(HardwareError::DeviceNotFound),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PassthruInfo {
    name: String,
    vendor: String,
    function_lib: String,
    can: bool,
    iso15765: bool,
    iso14230: bool,
    iso9141: bool,
    j1850pwm: bool,
    j1850vpw: bool,
    sci_a_engine: bool,
    sci_b_engine: bool,
    sci_a_trans: bool,
    sci_b_trans: bool
}

impl PassthruInfo {
    #[cfg(unix)]
    pub fn new(path: &std::path::PathBuf) -> HardwareResult<Self> {
        return if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(s.as_str()) {
                let lib = match json["FUNCTION_LIB"].as_str() {
                    Some(s) => shellexpand::tilde(s),
                    None => "UNKNOWN PASSTHRU DEVICE FUNCTION LIB".into(),
                };
                let name = match json["NAME"].as_str() {
                    Some(s) => s,
                    None => "UNKNOWN PASSTHRU DEVICE".into(),
                };
                let vend = match json["VENDOR"].as_str() {
                    Some(s) => s,
                    None => "UNKNOWN PASSTHRU DEVICE VENDOR".into(),
                };
                Ok(PassthruInfo {
                    function_lib: String::from(lib),
                    name: String::from(name),
                    vendor: String::from(vend),
                    can: Self::read_bool(&json, "CAN"),
                    iso15765: Self::read_bool(&json, "ISO15765"),
                    iso14230: Self::read_bool(&json, "ISO14230"),
                    iso9141: Self::read_bool(&json, "ISO9141"),
                    j1850pwm: Self::read_bool(&json, "J1850PWM"),
                    j1850vpw: Self::read_bool(&json, "J1850VPW"),
                    sci_a_engine: Self::read_bool(&json, "SCI_A_ENGINE"),
                    sci_a_trans: Self::read_bool(&json, "SCN_A_TRANS"),
                    sci_b_engine: Self::read_bool(&json, "SCI_B_ENGINE"),
                    sci_b_trans: Self::read_bool(&json, "SCI_B_TRANS"),
                })
            } else {
                return Err(HardwareError::DeviceNotFound);
            }
        } else {
            Err(HardwareError::DeviceNotFound)
        };
    }

    #[cfg(unix)]
    #[inline]
    fn read_bool(j: &serde_json::Value, s: &str) -> bool {
        j[s].as_bool().unwrap_or(false)
    }

    #[cfg(windows)]
    #[inline]
    fn read_bool(k: &RegKey, name: &str) -> bool {
        let val: u32 = match k.get_value(name.to_string()) {
            Ok(b) => b,
            Err(_) => return false,
        };
        return val != 0;
    }
}

impl Into<HardwareInfo> for &PassthruInfo {
    fn into(self) -> HardwareInfo {
        HardwareInfo {
            name: self.name.clone(),
            vendor: self.vendor.clone(),
            capabilities: HardwareCapabilities { 
                iso_tp: self.iso15765, 
                can: self.can, 
                kline: self.iso9141, 
                kline_kwp: self.iso14230, 
                sae_j1850: self.j1850pwm && self.j1850vpw, 
                sci: self.sci_a_engine && self.sci_a_trans && self.sci_b_engine && self.sci_b_trans, 
                ip: false // Passthru never supports this protocol
            }
        }
    }
}

/// Passthru device
#[derive(Debug, Clone)]
pub struct PassthruDevice {
    info: HardwareInfo,
    drv: PassthruDrv,
    device_idx: Option<u32>,
    can_channel: bool
}

impl PassthruDevice {
    /// Opens the passthru device
    fn open_device(info: &PassthruInfo) -> HardwareResult<Self> {
        let lib = info.function_lib.clone();
        let mut drv = lib_funcs::PassthruDrv::load_lib(lib)?;
        let idx = drv.open()?;
        Ok(Self {
            info: info.into(),
            drv,
            device_idx: Some(idx),
            can_channel: false
        })
    }

    pub (crate) fn safe_passthru_op<X, T: FnOnce(u32, PassthruDrv) -> lib_funcs::PassthruResult<X>>(&self, f: T) -> HardwareResult<X> {
        match self.device_idx {
            Some(idx) => match f(idx, self.drv.clone()) {
                Ok(res) => Ok(res),
                Err(e) => {
                    if e == PassthruError::ERR_FAILED { // Err failed, query the adapter for error!
                        if let Ok(reason) = self.drv.get_last_error() {
                            Err(HardwareError::APIError {
                                code: e as u32,
                                desc: reason
                            })
                        } else { // No reason, just ERR_FAILED
                            Err(e.into())
                        }
                    } else {
                        Err(e.into())
                    }
                }
            },
            None => Err(HardwareError::DeviceNotOpen)
        }
    }
}

impl Drop for PassthruDevice {
    #[allow(unused_must_use)] // If this function fails, then device is already closed, so don't care!
    fn drop(&mut self) {
        if let Some(idx) = self.device_idx {
            self.drv.close(idx);
            self.device_idx = None;
        }
    }
}

impl<'a> super::Hardware<'a> for PassthruDevice {
    fn create_iso_tp_channel(&'a mut self) -> HardwareResult<Box<&'a dyn IsoTPChannel>> {
        if !self.info.capabilities.iso_tp {
            return Err(HardwareError::ChannelNotSupported)
        }
        todo!()
    }

    fn create_can_channel(&'a mut self) -> HardwareResult<Box<&'a dyn CanChannel>> {
        if !self.info.capabilities.can {
            return Err(HardwareError::ChannelNotSupported)
        }
        if self.can_channel {
            return Err(HardwareError::ConflictingChannel)
        }
        let can_channel = PassthruCanChannel {
            device: self,
            channel_id: 0
        };

        Ok(Box::new(&can_channel))
    }

    fn read_battery_voltage(&mut self) -> Option<f32> {
        let mut output: u32 = 0;
        match self.safe_passthru_op(|idx, drv: PassthruDrv| {
            drv.ioctl(idx, IoctlID::READ_VBATT, std::ptr::null_mut(), (&mut output) as *mut _ as *mut c_void)
        }) {
            Ok(_) => Some(output as f32 / 1000.0),
            Err(_) => None
        }
    }

    fn get_capabilities(&self) -> &super::HardwareCapabilities {
        &self.info.capabilities
    }
}

/// Passthru device CAN Channel
#[derive(Debug)]
pub struct PassthruCanChannel<'a> {
    pub (crate) device: &'a mut PassthruDevice,
    pub (crate) channel_id: u32
}

impl<'a> CanChannel for PassthruCanChannel<'a> {
    fn open(&mut self, baud: u32, use_extended: bool) -> crate::channel::ChannelResult<()> {
        let mut flags: u32 = 0;
        if use_extended {
            flags |= ConnectFlags::CAN_29BIT_ID as u32;
        }
        match self.device.safe_passthru_op(|idx, device| {
            device.connect(idx, j2534_rust::Protocol::CAN, flags, baud)
        }) {
            Ok(channel) => {
                self.channel_id = channel;
                Ok(())
            },
            Err(e) => Err(ChannelError::HardwareError(e))
        }
    }
}

impl<'a> PacketChannel<CanFrame> for PassthruCanChannel<'a> {
    fn close(&mut self) -> crate::channel::ChannelResult<()> {
        self.device.safe_passthru_op(|_, device|{
            device.disconnect(self.channel_id)
        }).map_err(|e| e.into())
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, timeout_ms: u32) -> crate::channel::ChannelResult<()> {
        let mut msgs: Vec<PASSTHRU_MSG> = packets.iter().map(|f| f.into()).collect();
        self.device.safe_passthru_op(|_, device| {
            device.write_messages(self.channel_id, &mut msgs, timeout_ms)
        }).map_err(|e| e.into())
        .map(|_|())
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> crate::channel::ChannelResult<Vec<CanFrame>> {
        match self.device.safe_passthru_op(|_, device| {
            device.read_messages(self.channel_id, max as u32, timeout_ms)
        }) {
            Ok(res) => {
                Ok(res.iter().map(|pt| CanFrame::from(pt)).collect())
            },
            Err(e) => Err(e.into())
        }
    }

    fn clear_rx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        self.device.safe_passthru_op(|_, device| {
            device.ioctl(self.channel_id, IoctlID::CLEAR_RX_BUFFER, std::ptr::null_mut(), std::ptr::null_mut())
        }).map_err(|e| e.into())
    }

    fn clear_tx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        self.device.safe_passthru_op(|_, device| {
            device.ioctl(self.channel_id, IoctlID::CLEAR_TX_BUFFER, std::ptr::null_mut(), std::ptr::null_mut())
        }).map_err(|e| e.into())
    }
}

impl<'a> Drop for PassthruCanChannel<'a> {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        println!("CAN Channel drop called");
        self.close();
        self.device.can_channel = false;
    }
}

impl From<&CanFrame> for PASSTHRU_MSG {
    fn from(frame: &CanFrame) -> Self {
        let mut f = PASSTHRU_MSG::default();
        if frame.is_extended() {
            f.tx_flags |= TxFlag::CAN_29BIT_ID.bits();
        }
        f.data_size = (frame.get_data().len() + 4) as u32;
        f.data[0] = (frame.get_address() >> 24) as u8;
        f.data[1] = (frame.get_address() >> 16) as u8;
        f.data[2] = (frame.get_address() >> 8) as u8;
        f.data[3] = frame.get_address() as u8;
        f.data[4..4+frame.get_data().len()].copy_from_slice(frame.get_data());
        f
    }
}

impl From<&PASSTHRU_MSG> for CanFrame {
    fn from(msg: &PASSTHRU_MSG) -> CanFrame {
        let id = 
            (msg.data[0] as u32) << 24 |
            (msg.data[1] as u32) << 16 |
            (msg.data[2] as u32) << 8 |
            (msg.data[3] as u32);
        let data = &msg.data[4..msg.data_size as usize];
        let is_ext = msg.tx_flags & TxFlag::CAN_29BIT_ID.bits() != 0;
        CanFrame::new(id, data, is_ext)
    }
}


impl From<j2534_rust::PassthruError> for HardwareError {
    fn from(err: j2534_rust::PassthruError) -> Self {
        HardwareError::APIError {
            code: err as u32,
            desc: err.to_string().into(),
        }
    }
}

#[cfg(test)]
pub mod passthru_test {
    use crate::hardware::{Hardware, HardwareScanner};

    use super::*;

    #[test]
    pub fn scan_test() {
        let scanner = PassthruScanner::new();
        println!("{:#?}", scanner.list_devices());
        let mut device =  scanner.open_device_by_name("Macchina A0").unwrap();
        println!("ECU DIAG TEST ==> Loaded device: {:#?}", device);
        println!("ECU DIAG TEST ==> Battery voltage: {:?}", device.read_battery_voltage());
        let mut can_channel = device.create_can_channel().unwrap();
        can_channel.open(500000, false).unwrap();
        let packets = can_channel.read_packets(100, 0).unwrap();
        println!("{:?}", packets);
    }
}