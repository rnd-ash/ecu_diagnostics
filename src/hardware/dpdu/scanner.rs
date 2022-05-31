//! Module for scanning for PDU devices on a system

use std::{fs::File, io::Read};

use serde_xml_rs::from_str;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::hardware::{HardwareCapabilities, HardwareInfo, HardwareScanner};

use super::PDUDevice;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct MvciPduApiRoot {
    mvci_part2_standard_version: String,
    #[serde(rename = "$value")]
    api: Vec<MvciPduApi>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct Uri {
    uri: String,
}

impl Into<String> for Uri {
    fn into(self) -> String {
        self.uri.replace("file:///", "")
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct MvciPduApi {
    short_name: String,
    description: String,
    supplier_name: String,
    library_file: Uri,
    module_description_file: Uri,
    cable_description_file: Uri,
}

#[derive(Debug, Clone)]
/// PDU device scanner
pub struct PDUScanner {
    api_root_xml: Option<MvciPduApiRoot>,
}

impl PDUScanner {
    /// Creates a new PDU API scanner
    pub fn new() -> Self {
        // Windows only, 1. Find PDU Root XML
        #[cfg(windows)]
        match RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE\\WOW6432Node\\D-PDU API") // 64bit OS
            .or_else(|_| RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\D-PDU API")) // 32bit OS
            .and_then(|key| key.get_value::<String, _>("Root file"))
        {
            Ok(r) => {
                let mut buffer = String::new();
                let f = File::open(r)
                    .and_then(|mut f| f.read_to_string(&mut buffer))
                    .map(|_| buffer)
                    .ok()
                    .and_then(|x| from_str::<MvciPduApiRoot>(&x).ok());
                Self { api_root_xml: f }
            }
            Err(_) => Self { api_root_xml: None },
        }
        #[cfg(unix)]
        {
            let mut buffer = String::new();
            let f = File::open("/etc/pdu_api_root.xml")
                .and_then(|mut f| f.read_to_string(&mut buffer))
                .map(|_| buffer)
                .ok()
                .and_then(|x| from_str::<MvciPduApiRoot>(&x).ok());
            Self { api_root_xml: f }
        }
    }
}

impl HardwareScanner<PDUDevice> for PDUScanner {
    fn list_devices(&self) -> Vec<HardwareInfo> {
        if let Some(api) = &self.api_root_xml {
            let mut result = Vec::new();
            for device in &api.api {
                let hw_info = HardwareInfo {
                    name: device.description.clone(),
                    vendor: Some(device.supplier_name.clone()),
                    device_fw_version: None,
                    api_version: Some(api.mvci_part2_standard_version.clone()),
                    library_version: None,
                    library_location: Some(device.library_file.clone().into()),
                    capabilities: HardwareCapabilities {
                        iso_tp: false,
                        can: false,
                        kline: false,
                        kline_kwp: false,
                        sae_j1850: false,
                        sci: false,
                        ip: false,
                    },
                };
                result.push(hw_info);
            }
            result
        } else {
            Vec::new()
        }
    }

    fn open_device_by_index(
        &self,
        idx: usize,
    ) -> crate::hardware::HardwareResult<std::sync::Arc<std::sync::Mutex<PDUDevice>>> {
        todo!()
    }

    fn open_device_by_name(
        &self,
        name: &str,
    ) -> crate::hardware::HardwareResult<std::sync::Arc<std::sync::Mutex<PDUDevice>>> {
        todo!()
    }
}
