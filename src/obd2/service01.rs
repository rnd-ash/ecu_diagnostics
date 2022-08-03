//! OBD2 service 01 (Show current data)

use crate::obd2::data_pids::DataPid;
use crate::obd2::units::ObdValue;
use crate::obd2::{decode_pid_response, OBD2Cmd, OBD2Command, OBD2DiagnosticServer};
use crate::{DiagError, DiagServerResult};

#[derive(Debug)]
/// Service 01 wrapper for OBD
pub struct Service01<'a> {
    server: &'a mut OBD2DiagnosticServer,
    support_list: Vec<bool>,
}

impl OBD2DiagnosticServer {
    /// Initializes the service 01 wrapper. Automatically query's the ECU
    /// on init for supported PIDs.
    /// NOTE: Unlike other functions, if this function encounters a ECU communication
    /// error, it will still return OK.
    pub fn init_service_01(&mut self) -> DiagServerResult<Service01> {
        // Query supported pids
        let mut total_support_list = Vec::new();
        for i in (0..0xFF).step_by(0x20) {
            let x = self.exec_command(OBD2Cmd::new(OBD2Command::Service01, &[i as u8]));
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
            if total_support_list.last().unwrap() & 0x01 == 0 { // Early return if we don't support any more PIDs
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
    pub fn get_supported_pids(&self) -> Vec<DataPid> {
        let mut r = Vec::new();
        for (idx, supported) in self.support_list.iter().enumerate() {
            if *supported {
                let pid = (idx + 1) as u8;
                if !&[0x13, 0x1D, 0x20, 0x40, 0x60, 0x80, 0xA0, 0xC0, 0xE0].contains(&pid) {
                    r.push(DataPid::from(pid))
                }
            }
        }
        r
    }

    /// Query's a data PID from Service 01
    pub fn query_pid(&mut self, pid: DataPid) -> DiagServerResult<Vec<ObdValue>> {
        pid.get_value(self.server, None)
    }
}

#[cfg(test)]
pub mod service_09_test {
    use crate::channel::IsoTPSettings;
    use crate::hardware::socketcan::SocketCanScanner;
    use crate::hardware::Hardware;
    use crate::hardware::HardwareScanner;
    use crate::obd2::units::{ObdUnitType, ObdValue};
    use crate::obd2::{OBD2DiagnosticServer, Obd2ServerOptions};
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

    #[test]
    #[ignore] // Requires ECU to be present and SocketCAN interface!
    pub fn test_init() {
        let scan = SocketCanScanner::new().open_device_by_name("can0").unwrap();
        let socket = Hardware::create_iso_tp_channel(scan).unwrap();
        let mut obd = OBD2DiagnosticServer::new_over_iso_tp(
            Obd2ServerOptions {
                send_id: 0x07E0,
                recv_id: 0x07E8,
                read_timeout_ms: 500,
                write_timeout_ms: 500,
            },
            socket,
            IsoTPSettings {
                block_size: 8,
                st_min: 20,
                extended_addresses: None,
                pad_frame: true,
                can_speed: 500_000,
                can_use_ext_addr: false,
            },
        )
        .unwrap();
        obd.read_dtcs();
        let mut s_01 = obd.init_service_01().unwrap();
        let pids = s_01.get_supported_pids();
        println!("Supported PIDs: {:?}", pids);
        for p in pids {
            match s_01.query_pid(p) {
                Err(e) => println!("Query for {:?} failed {}", p, e),
                Ok(res) => {
                    for param in res {
                        println!("{}", param.to_string())
                    }
                }
            }
        }
    }
}
