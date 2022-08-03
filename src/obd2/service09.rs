//! OBD2 service 09 (Request vehicle information)

use crate::obd2::{decode_pid_response, OBD2Cmd, OBD2Command, OBD2DiagnosticServer};
use crate::{DiagError, DiagServerResult};

#[derive(Debug)]
/// Service 09 wrapper for OBD
pub struct Service09<'a> {
    server: &'a mut OBD2DiagnosticServer,
    support_list: Vec<bool>,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Service 09 PIDs
pub enum Service09Pid {
    /// VIN message count (Only for LIN)
    VinMsgCount,
    /// VIN
    Vin,
    /// Calibration ID message count (Only for LIN)
    CalibrationIDMsgCount,
    /// Calibration ID
    CalibrationID,
    /// CVN message count (Only for LIN)
    CvnMsgCount,
    /// CVN
    Cvn,
    /// In use performance tracking for spark ignition engines
    InUsePerfTracking,
    ///ECU name message count (Only for LIN)
    EcuNameMsgCount,
    /// ECU name
    EcuName,
    /// Unknown PID by the OBD spec, might be manufacturer specific
    Unknown(u8),
}

impl OBD2DiagnosticServer {
    /// Initializes the service 09 wrapper. Automatically query's the ECU
    /// on init for supported PIDs
    pub fn init_service_09(&mut self) -> DiagServerResult<Service09> {
        // Query supported pids
        let pid_support_list = self.exec_command(OBD2Cmd::new(OBD2Command::Service09, &[0x00]))?;
        Ok(Service09 {
            server: self,
            support_list: decode_pid_response(&pid_support_list[2..]),
        })
    }
}

impl<'a> Service09<'a> {
    /// Returns a list of supported PIDs supported by the ECU for service 01
    pub fn get_supported_sids(&self) -> Vec<Service09Pid> {
        let mut r = Vec::new();
        for (pid, supported) in self.support_list.iter().enumerate() {
            // Remember +1 as pid 0x00 is supported (Requested supported IDs
            if *supported {
                r.push(match pid + 1 {
                    0x01 => Service09Pid::VinMsgCount,
                    0x02 => Service09Pid::Vin,
                    0x03 => Service09Pid::CalibrationIDMsgCount,
                    0x04 => Service09Pid::CalibrationID,
                    0x05 => Service09Pid::CvnMsgCount,
                    0x06 => Service09Pid::Cvn,
                    0x08 => Service09Pid::InUsePerfTracking,
                    0x09 => Service09Pid::EcuNameMsgCount,
                    0x0A => Service09Pid::EcuName,
                    x => Service09Pid::Unknown(x as u8),
                })
            }
        }
        r
    }

    /// Reads the ECU's stored VIN
    pub fn read_vin(&mut self) -> DiagServerResult<String> {
        if !self.support_list[0x01] {
            return Err(DiagError::NotSupported); // Unsupported request
        }
        let resp = self
            .server
            .exec_command(OBD2Cmd::new(OBD2Command::Service09, &[0x02]))?;
        Ok(
            String::from_utf8_lossy(resp.get(3..).ok_or(DiagError::InvalidResponseLength)?)
                .into_owned(),
        )
    }

    /// Reads the vehicles stored calibration ID (More than 1 may be returned)
    pub fn read_calibration_id(&mut self) -> DiagServerResult<Vec<String>> {
        if !self.support_list[0x03] {
            return Err(DiagError::NotSupported); // Unsupported request
        }
        let mut resp = self
            .server
            .exec_command(OBD2Cmd::new(OBD2Command::Service09, &[0x04]))?;
        resp.drain(0..3);
        return Ok(resp
            .chunks(16)
            .map(|c| String::from_utf8_lossy(c).to_string())
            .collect());
    }

    /// Reads the vehicles stored calibration verification number (More than 1 may be returned)
    pub fn read_cvn(&mut self) -> DiagServerResult<Vec<String>> {
        if !self.support_list[0x05] {
            return Err(DiagError::NotSupported); // Unsupported request
        }
        let mut resp = self
            .server
            .exec_command(OBD2Cmd::new(OBD2Command::Service09, &[0x06]))?;
        resp.drain(0..3);
        return Ok(resp
            .chunks(4)
            .map(|c| format!("{:02X}{:02X}{:02X}{:02X}", c[0], c[1], c[2], c[3]))
            .collect());
    }
}

#[cfg(test)]
pub mod service_09_test {
    use crate::channel::IsoTPSettings;
    use crate::hardware::socketcan::SocketCanScanner;
    use crate::hardware::Hardware;
    use crate::hardware::HardwareScanner;
    use crate::obd2::{OBD2DiagnosticServer, Obd2ServerOptions};

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

        let mut s_09 = obd.init_service_09().unwrap();
        println!("Supported Service_09 Pids: {:?}", s_09.get_supported_sids());
        println!("VIN: {}", s_09.read_vin().unwrap());
        println!("CID(s): {:?}", s_09.read_calibration_id().unwrap());
        println!("CVN(s): {:?}", s_09.read_cvn().unwrap());
    }
}
