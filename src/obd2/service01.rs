//! OBD2 service 01 (Show current data)

use crate::dynamic_diag::DynamicDiagSession;
use crate::obd2::units::ObdValue;
use crate::obd2::{decode_pid_response, DataPidWrapper};
use crate::{DiagError, DiagServerResult};
use auto_uds::obd2::{DataPidByte, Obd2Command};

#[derive(Debug)]
/// Service 01 wrapper for OBD
pub struct Service01<'a> {
    server: &'a mut DynamicDiagSession,
    support_list: Vec<bool>,
}

impl DynamicDiagSession {
    /// Initializes the service 01 wrapper. Automatically query's the ECU
    /// on init for supported PIDs.
    /// NOTE: Unlike other functions, if this function encounters a ECU communication
    /// error, it will still return OK.
    pub fn obd_init_service_01(&mut self) -> DiagServerResult<Service01> {
        // Query supported pids
        let mut total_support_list = Vec::new();
        for i in (0..0xFF).step_by(0x20) {
            let x = self.send_command_with_response(Obd2Command::Service01, &[i as u8]);
            match x {
                Ok(resp) => total_support_list.extend_from_slice(&resp[2..]),
                Err(e) => {
                    if let DiagError::ECUError { code: _, def: _ } = e {
                        total_support_list.extend_from_slice(&[0x00, 0x00, 0x00, 0x00])
                    } else {
                        // Communication error
                        total_support_list.extend_from_slice(&[0x00, 0x00, 0x00, 0x00])
                    }
                }
            }
            if total_support_list.last().unwrap() & 0x01 == 0 {
                // Early return if we don't support any more PIDs
                break;
            }
        }
        Ok(Service01 {
            server: self,
            support_list: decode_pid_response(&total_support_list),
        })
    }
}

impl<'a> Service01<'a> {
    /// Returns a byte array of supported PIDs supported by the ECU for service 01
    pub fn get_supported_pids(&self) -> Vec<DataPidByte> {
        let mut r = Vec::new();
        for (idx, supported) in self.support_list.iter().enumerate() {
            if *supported {
                let pid = (idx + 1) as u8;
                if !&[0x13, 0x1D, 0x20, 0x40, 0x60, 0x80, 0xA0, 0xC0, 0xE0].contains(&pid) {
                    r.push(DataPidByte::from(pid))
                }
            }
        }
        r
    }

    /// Query's a data PID from Service 01
    pub fn query_pid(&mut self, pid: DataPidWrapper) -> DiagServerResult<Vec<ObdValue>> {
        pid.get_value(self.server, None)
    }
}

#[cfg(test)]
pub mod service_09_test {
    use crate::obd2::units::{ObdUnitType, ObdValue};
    use crate::DiagServerResult;

    fn print_pid(v: DiagServerResult<ObdValue>) {
        match v {
            Ok(value) => {
                if let ObdUnitType::Encoded(e) = value.get_value() {
                    println!("{} OK! Enum {}", value.get_name(), e)
                } else {
                    println!(
                        "{} OK! Metric: {}{}, Imperial: {}{}",
                        value.get_name(),
                        value.get_metric_data(),
                        value.get_metric_unit().unwrap_or_default(),
                        value.get_imperial_data(),
                        value.get_imperial_unit().unwrap_or_default()
                    );
                }
            }
            Err(e) => {
                eprintln!("PID request failed {}", e);
            }
        }
    }
}
