//! Module for scanning for PDU devices on a system

use crate::hardware::HardwareScanner;

use super::PDUDevice;



#[derive(Debug, Clone, Copy)]
/// PDU device scanner
pub struct PDUScanner {

}

impl HardwareScanner<PDUDevice> for PDUScanner {
    fn list_devices(&self) -> Vec<crate::hardware::HardwareInfo> {
        todo!()
    }

    fn open_device_by_index(&self, idx: usize) -> crate::hardware::HardwareResult<std::sync::Arc<std::sync::Mutex<PDUDevice>>> {
        todo!()
    }

    fn open_device_by_name(&self, name: &str) -> crate::hardware::HardwareResult<std::sync::Arc<std::sync::Mutex<PDUDevice>>> {
        todo!()
    }
}