//! The D-PDU API (Also known as ISO 22900-2) is a standard way for VCI devices to access and communicate with vehicles
//!
//! The API supports the following CAN based protocols:
//! * UDS / ISO14229
//! * DoCAN / ISO15765-3
//! * OBD / ISO15031
//! * KWP2000 over ISO15765
//! * KW1281 on VW TP1.6
//! * KWP2000 light plus on VW TP1.6
//! * KWP2000 light plus on VW TP2.0
//! * ISO11898 RAW
//!
//! It also supports the following K-Line based protocols:
//! * KWP2000 - ISO 14230-2/3
//! * OBD / ISO15031 - ISO15031-5 on ISO15031-4
//! * KW1281 on ISO 9141-2
//! * KWP2000 light plus VW on ISO 14230-2
//!
//! Additionally, the protocol also supports the following IP based protocols:
//! * ISO14229-5 on ISO 13400-2 (DoIP)
//!

use super::Hardware;

pub mod scanner;
pub mod lib_funcs;

#[derive(Debug)]
/// PDU Device
pub struct PDUDevice {
    lib: lib_funcs::PduDrv
}

impl Hardware for PDUDevice {
    fn create_iso_tp_channel(this: std::sync::Arc<std::sync::Mutex<Self>>) -> super::HardwareResult<Box<dyn crate::channel::IsoTPChannel>> {
        todo!()
    }

    fn create_can_channel(this: std::sync::Arc<std::sync::Mutex<Self>>) -> super::HardwareResult<Box<dyn crate::channel::CanChannel>> {
        todo!()
    }

    fn is_iso_tp_channel_open(&self) -> bool {
        todo!()
    }

    fn is_can_channel_open(&self) -> bool {
        todo!()
    }

    fn read_battery_voltage(&mut self) -> Option<f32> {
        todo!()
    }

    fn read_ignition_voltage(&mut self) -> Option<f32> {
        todo!()
    }

    fn get_info(&self) -> &super::HardwareInfo {
        todo!()
    }
}