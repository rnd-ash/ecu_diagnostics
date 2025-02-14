//! The Pass-thru API (Also known as SAE J2534) is an adapter protocol used by some vehicle communication
//! interfaces (VCI).
//!
//! This module provides support for Version 04.04 of the API, including experimental support for OSX and Linux, used by
//! [Macchina-J2534](http://github.com/rnd-ash/macchina-J2534)
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
//! querying the [HardwareCapabilities] matrix should be used to determine which protocols
//! are supported

use std::sync::atomic::{AtomicBool, Ordering};
use std::{ffi::c_void, time::Instant};

use std::sync::{Arc, Mutex, RwLock};

#[cfg(windows)]
use winreg::enums::*;

#[cfg(windows)]
use winreg::RegKey;

#[cfg(unix)]
use std::path::Path;

use j2534_rust::{
    ConnectFlags, FilterType, IoctlID, PassthruError, Protocol, RxFlag, TxFlag, PASSTHRU_MSG,
};

use crate::channel::{
    CanChannel, CanFrame, ChannelError, ChannelResult, IsoTPChannel, IsoTPSettings, Packet,
    PacketChannel, PayloadChannel,
};

use self::lib_funcs::PassthruDrv;

use super::{HardwareCapabilities, HardwareError, HardwareInfo, HardwareResult};

mod lib_funcs;
mod sw_isotp;

pub use sw_isotp::PtCombiChannel;

/// Device scanner for Passthru supported devices
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PassthruScanner {
    devices: Vec<PassthruInfo>,
}

impl Default for PassthruScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PassthruScanner {
    #[cfg(unix)]
    /// Creates a passthru scanner. Scanning is done
    /// by scanning the ~/.passthru folder on the users PC for supported passthru
    /// JSON entries. This is UNOFFICIAL and should be considered experimental
    /// as Passthru API does not out of the box support UNIX Operating systems.
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
                        .collect(),
                }
            }
            Err(_) => Self {
                devices: Vec::new(),
            },
        }
    }

    #[cfg(windows)]
    /// Creates a passthru scanner. This scanner scans for devices by checking
    /// the windows registry entry `HKEY_LOCAL_MACHINE\\SOFTWARE\\WOW6432Node\\PassthruSupport.04.04`
    pub fn new() -> Self {
        match RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\WOW6432Node\\PassThruSupport.04.04")
        {
            Ok(r) => Self {
                devices: r
                    .enum_keys()
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .map(|key| r.open_subkey(key))
                    .map(|x| PassthruInfo::new(&x.unwrap()))
                    .filter_map(|d| d.ok())
                    .collect(),
            },
            Err(_) => Self {
                devices: Vec::new(),
            },
        }
    }
}

impl super::HardwareScanner<PassthruDevice> for PassthruScanner {
    fn list_devices(&self) -> Vec<HardwareInfo> {
        self.devices.iter().map(|x| x.into()).collect()
    }

    fn open_device_by_index(&self, idx: usize) -> HardwareResult<PassthruDevice> {
        match self.devices.get(idx) {
            Some(info) => Ok(PassthruDevice::open_device(info)?),
            None => Err(HardwareError::DeviceNotFound),
        }
    }

    fn open_device_by_name(&self, name: &str) -> HardwareResult<PassthruDevice> {
        match self.devices.iter().find(|s| s.name == name) {
            Some(info) => Ok(PassthruDevice::open_device(info)?),
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
    sci_b_trans: bool,
}

impl PassthruInfo {
    #[cfg(unix)]
    pub fn new(path: &Path) -> HardwareResult<Self> {
        return if let Ok(s) = std::fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(s.as_str()) {
                let lib = json["FUNCTION_LIB"]
                    .as_str()
                    .unwrap_or("UNKNOWN PASSTHRU DEVICE FUNCTION LIB");
                let name = json["NAME"].as_str().unwrap_or("UNKNOWN PASSTHRU DEVICE");
                let vend = json["VENDOR"]
                    .as_str()
                    .unwrap_or("UNKNOWN PASSTHRU DEVICE VENDOR");
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
    pub fn new(r: &RegKey) -> HardwareResult<Self> {
        let lib: String = match r.get_value("FunctionLibrary") {
            Ok(s) => s,
            Err(_) => "UNKNOWN PASSTHRU DEVICE FUNCTION LIB".into(),
        };

        let name: String = match r.get_value("Name") {
            Ok(s) => s,
            Err(_) => "UNKNOWN PASSTHRU DEVICE".into(),
        };

        let vend: String = match r.get_value("Vendor") {
            Ok(s) => s,
            Err(_) => "UNKNOWN PASSTHRU DEVICE VENDOR".into(),
        };

        Ok(PassthruInfo {
            function_lib: String::from(lib),
            name: String::from(name),
            vendor: String::from(vend),
            can: Self::read_bool(&r, "CAN"),
            iso15765: Self::read_bool(&r, "ISO15765"),
            iso14230: Self::read_bool(&r, "ISO14230"),
            iso9141: Self::read_bool(&r, "ISO9141"),
            j1850pwm: Self::read_bool(&r, "J1850PWM"),
            j1850vpw: Self::read_bool(&r, "J1850VPW"),
            sci_a_engine: Self::read_bool(&r, "SCI_A_ENGINE"),
            sci_a_trans: Self::read_bool(&r, "SCN_A_TRANS"),
            sci_b_engine: Self::read_bool(&r, "SCI_B_ENGINE"),
            sci_b_trans: Self::read_bool(&r, "SCI_B_TRANS"),
        })
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

impl From<&PassthruInfo> for HardwareInfo {
    fn from(x: &PassthruInfo) -> Self {
        HardwareInfo {
            name: x.name.clone(),
            vendor: Some(x.vendor.clone()),
            device_fw_version: None,
            api_version: None,
            library_version: None,
            library_location: Some(x.function_lib.clone()),
            capabilities: HardwareCapabilities {
                iso_tp: x.iso15765,
                can: x.can,
                kline: x.iso9141,
                kline_kwp: x.iso14230,
                sae_j1850: x.j1850pwm && x.j1850vpw,
                sci: x.sci_a_engine && x.sci_a_trans && x.sci_b_engine && x.sci_b_trans,
                ip: false, // Passthru never supports this protocol
            },
        }
    }
}

/// Passthru device
#[derive(Clone, Debug)]
pub struct PassthruDevice {
    info: HardwareInfo,
    drv: Arc<Mutex<PassthruDrv>>,
    device_idx: Option<u32>,
    can_channel: Arc<AtomicBool>,
    isotp_channel: Arc<AtomicBool>,
    software_mode: Arc<AtomicBool>,
    connected_ok: Arc<AtomicBool>,
}

impl PassthruDevice {
    /// Opens the passthru device
    fn open_device(info: &PassthruInfo) -> HardwareResult<Self> {
        log::debug!(
            "Opening device {}. Function library is at {}",
            info.name,
            info.function_lib
        );
        let lib = info.function_lib.clone();
        let drv = Arc::new(Mutex::new(PassthruDrv::load_lib(lib)?));
        let mut lck = drv.lock().unwrap();
        let idx = lck.open()?;
        let mut ret = Self {
            info: info.into(),
            drv: drv.clone(),
            device_idx: Some(idx),
            can_channel: Arc::new(AtomicBool::new(false)),
            isotp_channel: Arc::new(AtomicBool::new(false)),
            software_mode: Arc::new(AtomicBool::new(false)),
            connected_ok: Arc::new(AtomicBool::new(true)),
        };
        if let Ok(version) = lck.get_version(idx) {
            // Set new version information from the device!
            ret.info.api_version = Some(version.api_version.clone());
            ret.info.device_fw_version = Some(version.fw_version.clone());
            ret.info.library_version = Some(version.dll_version);
        }
        drop(lck);
        Ok(ret)
    }

    pub(crate) fn safe_passthru_op<
        X,
        T: FnOnce(u32, &mut PassthruDrv) -> lib_funcs::PassthruResult<X>,
    >(
        &mut self,
        f: T,
    ) -> HardwareResult<X> {
        let mut lck = self.drv.lock().unwrap();
        match self.device_idx {
            Some(idx) => match f(idx, &mut lck) {
                Ok(res) => {
                    self.connected_ok.store(true, Ordering::Relaxed);
                    Ok(res)
                }
                Err(e) => {
                    log::warn!(
                        "Function failed with status {e:?}, status 0x{:02X}",
                        e as u32
                    );
                    if e == PassthruError::ERR_DEVICE_NOT_CONNECTED {
                        self.connected_ok.store(false, Ordering::Relaxed);
                        Err(e.into())
                    } else if e == PassthruError::ERR_FAILED {
                        // Err failed, query the adapter for error!
                        if let Ok(reason) = lck.get_last_error() {
                            log::warn!("Function generic failure reason: {reason}");
                            Err(HardwareError::APIError {
                                code: e as u32,
                                desc: reason,
                            })
                        } else {
                            log::warn!("Function generic failure with no reason");
                            // No reason, just ERR_FAILED
                            Err(e.into())
                        }
                    } else {
                        Err(e.into())
                    }
                }
            },
            None => Err(HardwareError::DeviceNotOpen),
        }
    }

    /// Toggles software channel emulation.
    /// This is useful if you are trying to run a concurrent ISO-TP and CAN channel. As
    /// the J2534 API does not support this (Confliting channels). Instead, the ISO-TP channel
    /// will be emulated in software and the CAN packets are manually sent over an opened CAN channel.
    ///
    /// The CAN channel can then be accessed as normal, in parallel!
    ///
    /// ## CAUTION
    /// Since this relys on a fully unfiltered CAN channel, some cheaper J2534 devices may struggle with
    /// this mode, and result in unstable ISO-TP connections!
    ///  
    /// Note that this toggled state will only affect newly created channels, currently
    /// opened channels will not be affected, so it would be best to close them prior to
    /// toggling this function
    pub fn toggle_sw_channel(&mut self, state: bool) {
        self.software_mode.store(state, Ordering::Relaxed);
    }

    /// Do it on raw device
    pub fn toggle_sw_channel_raw(dev: &mut Arc<RwLock<PassthruDevice>>, state: bool) {
        dev.write().unwrap().toggle_sw_channel(state)
    }

    pub(crate) fn make_can_channel_raw(&mut self) -> HardwareResult<PassthruCanChannel> {
        {
            if !self.info.capabilities.can {
                return Err(HardwareError::ChannelNotSupported);
            }
            if self.can_channel.load(Ordering::Relaxed)
                && !self.software_mode.load(Ordering::Relaxed)
            {
                return Err(HardwareError::ConflictingChannel);
            }
        }
        Ok(PassthruCanChannel {
            device: self.clone(),
            channel_id: None,
            baud: 0,
            use_ext: false,
        })
    }
}

impl Drop for PassthruDevice {
    #[allow(unused_must_use)] // If this function fails, then device is already closed, so don't care!
    fn drop(&mut self) {
        if Arc::strong_count(&self.drv) == 1 {
            log::debug!("Drop called for device");
            if let Some(idx) = self.device_idx {
                self.drv.lock().unwrap().close(idx);
                self.device_idx = None;
            }
        }
    }
}

impl super::Hardware for PassthruDevice {
    fn create_iso_tp_channel(&mut self) -> HardwareResult<Box<dyn IsoTPChannel>> {
        // If in sw mode
        if self.software_mode.load(Ordering::Relaxed) {
            return Ok(Box::new(PtCombiChannel::new(self.clone())?));
        }
        // Not in software mode
        {
            if !self.info.capabilities.iso_tp {
                return Err(HardwareError::ChannelNotSupported);
            }
            if self.can_channel.load(Ordering::Relaxed) {
                return Err(HardwareError::ConflictingChannel);
            }
        }
        let iso_tp_channel = PassthruIsoTpChannel {
            device: self.clone(),
            channel_id: None,
            cfg: IsoTPSettings::default(),
            ids: (0, 0),
            cfg_complete: false,
        };
        Ok(Box::new(iso_tp_channel))
    }

    fn create_can_channel(&mut self) -> HardwareResult<Box<dyn CanChannel>> {
        // If in sw mode
        if self.software_mode.load(Ordering::Relaxed) {
            return Ok(Box::new(PtCombiChannel::new(self.clone())?));
        }
        let can_channel = self.make_can_channel_raw()?;
        Ok(Box::new(can_channel))
    }

    fn is_iso_tp_channel_open(&self) -> bool {
        self.can_channel.load(Ordering::Relaxed)
    }

    fn is_can_channel_open(&self) -> bool {
        self.isotp_channel.load(Ordering::Relaxed)
    }

    #[allow(trivial_casts)]
    fn read_battery_voltage(&mut self) -> Option<f32> {
        let mut output: u32 = 0;
        match self.safe_passthru_op(|idx, drv: &mut PassthruDrv| {
            drv.ioctl(
                idx,
                IoctlID::READ_VBATT,
                std::ptr::null_mut(),
                (&mut output) as *mut _ as *mut c_void,
            )
        }) {
            Ok(_) => Some(output as f32 / 1000.0),
            Err(_) => None,
        }
    }

    #[allow(trivial_casts)]
    fn read_ignition_voltage(&mut self) -> Option<f32> {
        let mut output: u32 = 0;
        match self.safe_passthru_op(|idx, drv: &mut PassthruDrv| {
            drv.ioctl(
                idx,
                IoctlID::READ_PROG_VOLTAGE,
                std::ptr::null_mut(),
                (&mut output) as *mut _ as *mut c_void,
            )
        }) {
            Ok(_) => Some(output as f32 / 1000.0),
            Err(_) => None,
        }
    }

    fn get_info(&self) -> &HardwareInfo {
        &self.info
    }

    fn is_connected(&self) -> bool {
        self.connected_ok.load(Ordering::Relaxed)
    }
}

/// Passthru device CAN Channel
#[derive(Debug)]
pub struct PassthruCanChannel {
    pub(crate) device: PassthruDevice,
    pub(crate) channel_id: Option<u32>,
    pub(crate) baud: u32,
    pub(crate) use_ext: bool,
}

impl PassthruCanChannel {
    fn get_channel_id(&self) -> ChannelResult<u32> {
        match self.channel_id {
            None => Err(ChannelError::InterfaceNotOpen),
            Some(x) => Ok(x),
        }
    }
}

impl CanChannel for PassthruCanChannel {
    fn set_can_cfg(&mut self, baud: u32, use_extended: bool) -> ChannelResult<()> {
        self.baud = baud;
        self.use_ext = use_extended;
        Ok(())
    }
}

impl PacketChannel<CanFrame> for PassthruCanChannel {
    fn open(&mut self) -> ChannelResult<()> {
        // Already open, ignore request
        if self.channel_id.is_some() {
            return Ok(());
        }

        let mut flags = ConnectFlags::empty();
        if self.use_ext {
            flags |= ConnectFlags::CAN_29BIT_ID;
        }
        // Initialize the interface
        let channel_id = self
            .device
            .safe_passthru_op(|device_id, device| {
                device.connect(device_id, Protocol::CAN, flags.bits(), self.baud)
            })
            .map_err(ChannelError::HardwareError)?;
        self.device.can_channel.store(true, Ordering::Relaxed); // Acknowledge CAN is open now
        self.channel_id = Some(channel_id);

        // Now create open filter
        let mut mask = PASSTHRU_MSG {
            protocol_id: Protocol::CAN as u32,
            data_size: 4,
            ..Default::default()
        };

        let mut pattern = PASSTHRU_MSG {
            protocol_id: Protocol::CAN as u32,
            data_size: 4,
            ..Default::default()
        };

        // For open filter,
        // mask and pattern both need to be 0x0000
        //
        // CANID & 0x0000 == 0x0000
        mask.data[0..4].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        pattern.data[0..4].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        match self.device.safe_passthru_op(|_, device| {
            device.start_msg_filter(channel_id, FilterType::PASS_FILTER, &mask, &pattern, None)
        }) {
            Ok(_) => Ok(()), // Channel setup complete
            Err(e) => {
                // Oops! Teardown
                if let Err(e) = self.close() {
                    log::debug!("TODO PT close failed! {e}")
                }
                Err(e.into())
            }
        }
    }
    fn close(&mut self) -> ChannelResult<()> {
        // Channel already closed, ignore request
        if self.channel_id.is_none() {
            return Ok(());
        }
        let id = self.get_channel_id().unwrap(); // Unwrap as we checked previously if none
        self.device
            .safe_passthru_op(|_, device| device.disconnect(id))
            .map_err(ChannelError::HardwareError)?;
        self.device.can_channel.store(false, Ordering::Relaxed);
        self.channel_id = None;
        Ok(())
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, timeout_ms: u32) -> ChannelResult<()> {
        let channel_id = self.get_channel_id()?;
        let mut msgs: Vec<PASSTHRU_MSG> = packets.iter().map(|f| f.into()).collect();
        self.device
            .safe_passthru_op(|_, device| device.write_messages(channel_id, &mut msgs, timeout_ms))
            .map_err(ChannelError::HardwareError)?;
        Ok(())
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> ChannelResult<Vec<CanFrame>> {
        let channel_id = self.get_channel_id()?;
        match self
            .device
            .safe_passthru_op(|_, device| device.read_messages(channel_id, max as u32, timeout_ms))
        {
            Ok(res) => Ok(res.iter().map(CanFrame::from).collect()),
            Err(e) => Err(e.into()),
        }
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        let channel_id = self.get_channel_id()?;
        self.device
            .safe_passthru_op(|_, device| {
                device.ioctl(
                    channel_id,
                    IoctlID::CLEAR_RX_BUFFER,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            })
            .map_err(|e| e.into())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        let channel_id = self.get_channel_id()?;
        self.device
            .safe_passthru_op(|_, device| {
                device.ioctl(
                    channel_id,
                    IoctlID::CLEAR_TX_BUFFER,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            })
            .map_err(|e| e.into())
    }
}

impl Drop for PassthruCanChannel {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        log::debug!("Drop called for CanChannel");
        // Close the channel before dropping!
        self.close();
    }
}

/// Passthru ISO TP Channel
#[derive(Debug)]
pub struct PassthruIsoTpChannel {
    device: PassthruDevice,
    channel_id: Option<u32>,
    cfg: IsoTPSettings,
    ids: (u32, u32),
    cfg_complete: bool,
}

impl PassthruIsoTpChannel {
    fn get_channel_id(&self) -> ChannelResult<u32> {
        match self.channel_id {
            None => Err(ChannelError::InterfaceNotOpen),
            Some(x) => Ok(x),
        }
    }
}

impl PayloadChannel for PassthruIsoTpChannel {
    fn open(&mut self) -> ChannelResult<()> {
        if self.channel_id.is_some() {
            return Ok(());
        }
        let mut flags = ConnectFlags::empty();
        if self.cfg.can_use_ext_addr {
            flags |= ConnectFlags::CAN_29BIT_ID;
        }
        if self.cfg.extended_addresses.is_some() {
            flags |= ConnectFlags::ISO15765_ADDR_TYPE;
        }

        // Initialize the interface
        let channel_id = self
            .device
            .safe_passthru_op(|device_id, device| {
                device.connect(
                    device_id,
                    Protocol::ISO15765,
                    flags.bits(),
                    self.cfg.can_speed,
                )
            })
            .map_err(ChannelError::HardwareError)?;
        self.device.isotp_channel.store(true, Ordering::Relaxed); // Acknowledge CAN is open now
        self.channel_id = Some(channel_id);

        // Now create open filter
        let mut mask = PASSTHRU_MSG {
            protocol_id: Protocol::ISO15765 as u32,
            data_size: if self.cfg.extended_addresses.is_some() {
                5
            } else {
                4
            },
            ..Default::default()
        };

        let mut pattern = PASSTHRU_MSG {
            protocol_id: Protocol::ISO15765 as u32,
            data_size: if self.cfg.extended_addresses.is_some() {
                5
            } else {
                4
            },
            ..Default::default()
        };

        let mut flow_control = PASSTHRU_MSG {
            protocol_id: Protocol::ISO15765 as u32,
            data_size: if self.cfg.extended_addresses.is_some() {
                5
            } else {
                4
            },
            ..Default::default()
        };

        // 3 filters are to be configured like so:
        // Mask: 0xFFFF
        // Pattern: Recv ID
        // FC: Send ID
        match self.cfg.extended_addresses {
            Some((tx, rx)) => {
                mask.data[0..5].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
                pattern.data[0] = rx;
                pattern.data[1..4].copy_from_slice(&self.ids.1.to_be_bytes());
                flow_control.data[0] = tx;
                flow_control.data[1..4].copy_from_slice(&self.ids.0.to_be_bytes());
            }
            None => {
                mask.data[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
                pattern.data[0..4].copy_from_slice(&self.ids.1.to_be_bytes());
                flow_control.data[0..4].copy_from_slice(&self.ids.0.to_be_bytes());
            }
        }

        match self.device.safe_passthru_op(|_, device| {
            device.start_msg_filter(
                channel_id,
                FilterType::FLOW_CONTROL_FILTER,
                &mask,
                &pattern,
                Some(flow_control),
            )
        }) {
            Ok(_) => Ok(()), // Channel setup complete
            Err(e) => {
                // Oops! Teardown
                if let Err(e) = self.close() {
                    log::debug!("TODO PT close failed! {e}")
                }
                Err(e.into())
            }
        }
    }

    fn close(&mut self) -> ChannelResult<()> {
        // Channel already closed, ignore request
        if self.channel_id.is_none() {
            return Ok(());
        }
        {
            let id = self.get_channel_id().unwrap(); // Unwrap as we checked previously if none
            self.device
                .safe_passthru_op(|_, device| device.disconnect(id))
                .map_err(ChannelError::HardwareError)?;
            self.device.isotp_channel.store(false, Ordering::Relaxed);
            self.channel_id = None;
        }
        Ok(())
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        self.ids = (send, recv);
        Ok(())
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        let channel_id = self.get_channel_id()?;
        let start = Instant::now();
        let timeout = std::cmp::max(1, timeout_ms); // Need 1ms minimum
        while start.elapsed().as_millis() <= timeout as u128 {
            let read = self
                .device
                .safe_passthru_op(|_, device| device.read_messages(channel_id, 1, timeout_ms))
                .map_err(ChannelError::HardwareError)?;
            if let Some(msg) = read.get(0) {
                // Messages with these RxStatus bits sets are considered
                // to be either echo messages or indication of more data to be received
                // therefore, we ignore them
                //
                // This is a quirk fix specifically for some *cough* crappy *cough* VCI adapters
                // Normally, ISO15765_FIRST_FRAME are ALWAYS 4 bytes in length, but some of these adapters
                // don't do that, instead returning a message with an arbitrary number
                // of bytes, all set to 0x00. This breaks the specification!
                // but instead they use these 2 flags to denote echo messages!
                if (msg.rx_status & RxFlag::ISO15765_FIRST_FRAME.bits() == 0) // Not a first frame indication
                    && (msg.rx_status & RxFlag::TX_MSG_TYPE.bits() == 0)
                // Not an echo message
                {
                    // Normal way of checking for ISO15765_FIRST_FRAME indication
                    // Read complete!
                    // First 4 bytes are CAN ID, so ignore those
                    return Ok(msg.data[4..msg.data_size as usize].to_vec());
                }
            }
        }
        if timeout_ms == 0 {
            Err(ChannelError::BufferEmpty)
        } else {
            Err(ChannelError::ReadTimeout)
        }
    }

    fn write_bytes(
        &mut self,
        addr: u32,
        ext_id: Option<u8>,
        buffer: &[u8],
        timeout_ms: u32,
    ) -> ChannelResult<()> {
        let channel_id = self.get_channel_id()?;
        let mut write_msg = PASSTHRU_MSG {
            protocol_id: Protocol::ISO15765 as u32,
            data_size: 4 + buffer.len() as u32, // First 4 bytes are CAN ID
            ..Default::default()
        };

        let mut ext_addresses = self.cfg.extended_addresses;
        if let Some(id) = ext_id {
            log::warn!("extended_addresses was specified byt ext_id also was. ext_id overriding");
            ext_addresses = Some((id, 0));
        }

        let mut tx_flags = 0u32;
        if self.cfg.can_use_ext_addr {
            tx_flags |= TxFlag::CAN_29BIT_ID.bits();
        }
        if self.cfg.pad_frame {
            tx_flags |= TxFlag::ISO15765_FRAME_PAD.bits();
        }
        if ext_addresses.is_some() {
            tx_flags |= TxFlag::ISO15765_EXT_ADDR.bits();
        }
        let mut offset = 0;
        if let Some((tx, _rx)) = ext_addresses {
            offset = 1;
            write_msg.data[0] = tx;
        }
        write_msg.tx_flags = tx_flags;
        write_msg.data[offset] = (addr >> 24) as u8;
        write_msg.data[offset + 1] = (addr >> 16) as u8;
        write_msg.data[offset + 2] = (addr >> 8) as u8;
        write_msg.data[offset + 3] = addr as u8;
        write_msg.data[offset + 4..4 + buffer.len()].copy_from_slice(buffer);

        // Now transmit our message!

        self.device
            .safe_passthru_op(|_, device| {
                device.write_messages(channel_id, &mut [write_msg], timeout_ms)
            })
            .map_err(ChannelError::HardwareError)?;
        Ok(())
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        let channel_id = self.get_channel_id()?;
        self.device
            .safe_passthru_op(|_, device| {
                device.ioctl(
                    channel_id,
                    IoctlID::CLEAR_RX_BUFFER,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            })
            .map_err(|e| e.into())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        let channel_id = self.get_channel_id()?;
        self.device
            .safe_passthru_op(|_, device| {
                device.ioctl(
                    channel_id,
                    IoctlID::CLEAR_TX_BUFFER,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            })
            .map_err(|e| e.into())
    }
}

impl IsoTPChannel for PassthruIsoTpChannel {
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()> {
        self.cfg_complete = true;
        self.cfg = cfg;
        Ok(())
    }
}

impl Drop for PassthruIsoTpChannel {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        log::debug!("Drop called for IsoTPChannel");
        // Close the channel before dropping!
        self.close();
    }
}

impl From<&CanFrame> for PASSTHRU_MSG {
    fn from(frame: &CanFrame) -> Self {
        let mut f = PASSTHRU_MSG::default();
        if frame.is_extended() {
            f.tx_flags |= TxFlag::CAN_29BIT_ID.bits();
        }
        f.protocol_id = Protocol::CAN as u32;
        f.data_size = (frame.get_data().len() + 4) as u32;
        f.data[0..4].copy_from_slice(&frame.get_address().to_be_bytes());
        f.data[4..4 + frame.get_data().len()].copy_from_slice(frame.get_data());
        f
    }
}

impl From<&PASSTHRU_MSG> for CanFrame {
    fn from(msg: &PASSTHRU_MSG) -> CanFrame {
        CanFrame::new(
            u32::from_be_bytes(msg.data[0..4].try_into().unwrap()),
            &msg.data[4..msg.data_size as usize],
            msg.tx_flags & TxFlag::CAN_29BIT_ID.bits() != 0,
        )
    }
}

impl From<PassthruError> for HardwareError {
    fn from(err: PassthruError) -> Self {
        HardwareError::APIError {
            code: err as u32,
            desc: err.to_string(),
        }
    }
}
