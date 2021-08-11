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

use super::{HardwareCapabilities, HardwareInfo, HardwareResult};


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

    fn open_device_by_index(&mut self, idx: usize) -> super::HardwareResult<PassthruDevice> {
        todo!()
    }

    fn open_device_by_name(&mut self, name: &str) -> super::HardwareResult<PassthruDevice> {
        todo!()
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
        use super::HardwareError;

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
                ip: false // Passthru never supports this
            }
        }
    }
}

/// Passthru device
#[derive(Debug, Clone)]
pub struct PassthruDevice {

}

impl super::Hardware for PassthruDevice {
    fn to_iso_tp_channel(&mut self) -> Box<dyn crate::channel::IsoTPChannel> {
        todo!()
    }

    fn to_can_channel(&mut self) -> Box<dyn crate::channel::CanChannel> {
        todo!()
    }

    fn read_battery_voltage(&mut self) -> Option<f32> {
        todo!()
    }

    fn get_capabilities(&self) -> &super::HardwareCapabilities {
        todo!()
    }
}

#[cfg(test)]
pub mod passthru_test {
    use crate::hardware::HardwareScanner;

    use super::*;

    #[test]
    pub fn scan_test() {
        let scanner = PassthruScanner::new();
        println!("{:#?}", scanner.list_devices())
    }
}