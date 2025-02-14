//! Functions for reading DTCs from ECU

use crate::{
    dtc::{DTCFormatType, DTCStatus, DTC},
    dynamic_diag::DynamicDiagSession,
    DiagError, DiagServerResult,
};
use automotive_diag::kwp2000::{KwpCommand, KwpError, KwpErrorByte};
use automotive_diag::ByteWrapper::Standard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]

/// Represents a range of DTCs to query from the ECU
///
/// DTC Range support matrix
///
/// | DTCRange | Support by ECUs |
/// |--|--|
/// |[DTCRange::Powertrain] | Optional |
/// |[DTCRange::Chassis] | Optional |
/// |[DTCRange::Body] | Optional |
/// |[DTCRange::Network] | Optional |
/// |[DTCRange::All] | Mandatory |
pub enum DTCRange {
    /// All powertrain related DTCs
    Powertrain,
    /// All Chassis related DTCs
    Chassis,
    /// All Body related DTCs
    Body,
    /// All Network related DTCs
    Network,
    /// All DTCs from all groups
    All,
}

impl DTCRange {
    pub(crate) fn as_args(&self, pid: u8) -> [u8; 3] {
        match self {
            DTCRange::Powertrain => [pid, 0x00, 0x00],
            DTCRange::Chassis => [pid, 0x40, 0x00],
            DTCRange::Body => [pid, 0x80, 0x00],
            DTCRange::Network => [pid, 0xC0, 0x00],
            DTCRange::All => [pid, 0xFF, 0x00],
        }
    }
}

impl DynamicDiagSession {
    /// Returns a list of stored DTCs on the ECU in ISO15031-6 format
    pub fn kwp_read_stored_dtcs_iso15031(&self, range: DTCRange) -> DiagServerResult<Vec<DTC>> {
        let mut res = self.send_command_with_response(
            KwpCommand::ReadDiagnosticTroubleCodesByStatus,
            &range.as_args(0x00),
        )?;
        if res.len() < 5 {
            // No DTCs stored
            return Ok(Vec::new());
        }
        let num_dtcs = res[1];
        res.drain(0..2); // Remove everything up to the first DTC
        if res.len() % 3 != 0 {
            // Each DTC is 3 bytes, so this should divide by 0 if ECU response is valid
            return Err(DiagError::InvalidResponseLength);
        }

        let mut ret: Vec<DTC> = Vec::with_capacity(num_dtcs as usize); // Pre-allocate

        for x in (0..res.len()).step_by(3) {
            let status = res[x + 2];
            ret.push(DTC {
                format: DTCFormatType::Iso15031_6,
                raw: (res[x] as u32) << 8 | res[x + 1] as u32,
                status: DTCStatus::from_kwp_status(status),
                mil_on: status & 0b10000000 != 0,
                readiness_flag: status & 0b00010000 != 0,
            })
        }
        Ok(ret)
    }

    /// Returns a list of all supported DTCs on the ECU regardless of their status in ISO15031-6 format
    pub fn kwp_read_supported_dtcs_iso15031(&self, range: DTCRange) -> DiagServerResult<Vec<DTC>> {
        let res: Vec<DTC> = Vec::new();

        loop {
            let _res_bytes = self.send_command_with_response(
                KwpCommand::ReadDiagnosticTroubleCodesByStatus,
                &range.as_args(0x01),
            )?;
            match self.kwp_read_extended_supported_dtcs(range) {
                Ok(0) | Err(_) => break, // No more DTCs or error reading
                Ok(_) => {}
            }
        }

        Ok(res)
    }

    /// Returns a list of stored DTCs on the ECU in KWP2000 format
    pub fn kwp_read_stored_dtcs(&self, range: DTCRange) -> DiagServerResult<Vec<DTC>> {
        let mut res = self.send_command_with_response(
            KwpCommand::ReadDiagnosticTroubleCodesByStatus,
            &range.as_args(0x02),
        )?;
        if res.len() < 5 {
            // No DTCs stored
            return Ok(Vec::new());
        }
        let num_dtcs = res[1];
        res.drain(0..2); // Remove everything up to the first DTC
        if res.len() % 3 != 0 {
            // Each DTC is 3 bytes, so this should divide by 0 if ECU response is valid
            return Err(DiagError::InvalidResponseLength);
        }

        let mut ret: Vec<DTC> = Vec::with_capacity(num_dtcs as usize); // Pre-allocate

        for x in (0..res.len()).step_by(3) {
            let status = res[x + 2];
            ret.push(DTC {
                format: DTCFormatType::TwoByteHexKwp,
                raw: (res[x] as u32) << 8 | res[x + 1] as u32,
                status: DTCStatus::from_kwp_status(status),
                mil_on: status & 0b10000000 != 0,
                readiness_flag: status & 0b00010000 != 0,
            })
        }
        Ok(ret)
    }

    /// Returns a list of all supported DTCs on the ECU regardless of their status, in KWP2000 format.
    ///
    /// NOTE: Internally, this function will call [Kwp2000DiagnosticServer::read_extended_supported_dtcs] in a loop in order
    /// to read all DTCs regardless of transport layer limitations
    pub fn kwp_read_supported_dtcs(&self, range: DTCRange) -> DiagServerResult<Vec<DTC>> {
        let mut res: Vec<DTC> = Vec::new();
        loop {
            let mut res_bytes = self.send_command_with_response(
                KwpCommand::ReadDiagnosticTroubleCodesByStatus,
                &range.as_args(0x03),
            )?;

            if res_bytes.len() < 5 {
                // No DTCs stored
                return Ok(Vec::new());
            }
            res_bytes.drain(0..2); // Remove everything up to the first DTC
            if res_bytes.len() % 3 != 0 {
                // Each DTC is 3 bytes, so this should divide by 0 if ECU response is valid
                return Err(DiagError::InvalidResponseLength);
            }

            for x in (0..res_bytes.len()).step_by(3) {
                let status = res_bytes[x + 2];
                res.push(DTC {
                    format: DTCFormatType::TwoByteHexKwp,
                    raw: (res_bytes[x] as u32) << 8 | res_bytes[x + 1] as u32,
                    status: DTCStatus::from_kwp_status(status),
                    mil_on: status & 0b10000000 != 0,
                    readiness_flag: status & 0b00010000 != 0,
                })
            }
            match self.kwp_read_extended_supported_dtcs(range) {
                Ok(x) => {
                    if x == 0 || x as usize == res.len() {
                        return Ok(res); // Completed reading!
                    }
                    // Else keep looping to read DTCs
                }
                Err(_) => return Ok(res), // Return what we have
            }
        }
    }

    /// Asks the ECU to report its most recent DTCs that has been stored.
    /// Only one DTC is returned if stored, otherwise no DTC is returned.
    pub fn kwp_get_most_recent_dtc(&self, range: DTCRange) -> DiagServerResult<Option<DTC>> {
        let req = self.send_command_with_response(
            KwpCommand::ReadDiagnosticTroubleCodesByStatus,
            &range.as_args(0x04),
        )?;
        todo!("ECU Response: {:02X?}", req)
    }

    /// Upon execution of [Kwp2000DiagnosticServer::read_supported_dtcs] or [Kwp2000DiagnosticServer::read_supported_dtcs_iso15031],
    /// if the transport layer restricts the number of DTCs that can be read, or the number of DTCs exceeds 255,
    /// then this function will return the number of remaining supported of DTCs to read. [Kwp2000DiagnosticServer::read_supported_dtcs] or [Kwp2000DiagnosticServer::read_supported_dtcs_iso15031]
    /// should be executed to read the rest of the DTCs again within the ECUs P3-MAX time window
    pub fn kwp_read_extended_supported_dtcs(&self, range: DTCRange) -> DiagServerResult<u16> {
        match self.send_command_with_response(
            KwpCommand::ReadDiagnosticTroubleCodesByStatus,
            &range.as_args(0xE0),
        ) {
            Ok(x) => {
                if x.len() == 3 {
                    Ok((x[1] as u16) << 8 | x[2] as u16)
                } else {
                    Ok(0)
                }
            }
            Err(e) => {
                if let DiagError::ECUError { code, def } = e {
                    // ECU error, check if sub function not supported, in which case just return 0!
                    if KwpErrorByte::from(code)
                        == Standard(KwpError::SubFunctionNotSupportedInvalidFormat)
                    {
                        Ok(0)
                    } else {
                        Err(DiagError::ECUError { code, def })
                    }
                } else {
                    Err(e)
                }
            }
        }
    }
}
