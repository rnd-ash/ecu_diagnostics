//! Functions for reading DTCs from ECU

use crate::{DiagError, DiagServerResult, dtc::{DTC, DTCFormatType, DTCStatus}};

use super::{KWP2000Command, Kwp2000DiagnosticServer};

/// Represents a range of DTCs to read from the ECU
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    All
}

impl DTCRange {
    pub (crate) fn to_args(&self, pid: u8) -> [u8; 3] {
        match self {
            DTCRange::Powertrain => [pid, 0x00, 0x00],
            DTCRange::Chassis => [pid, 0x40, 0x00],
            DTCRange::Body => [pid, 0x80, 0x00],
            DTCRange::Network => [pid, 0xC0, 0x00],
            DTCRange::All => [pid, 0xFF, 0x00],
        }
    }
}

const KWP_DTC_FMT: DTCFormatType = crate::dtc::DTCFormatType::ISO15031_6;


/// Returns a list of all stored DTCs that the ECU has flagged
pub fn read_stored_dtcs(server: &mut Kwp2000DiagnosticServer, range: DTCRange) -> DiagServerResult<Vec<DTC>> {
    let mut res = server.execute_command_with_response(KWP2000Command::ReadDiagnosticTroubleCodesByStatus, &range.to_args(0x00))?;
    if res.len() < 5 { // No DTCs stored
        return Ok(Vec::new())
    }
    let num_dtcs = res[2];
    res.drain(0..3); // Remove everything up to the first DTC
    if res.len() % 3 != 0 { // Each DTC is 3 bytes, so this should divide by 0 if ECU response is valid
        return Err(DiagError::InvalidResponseLength)
    }

    let mut ret: Vec<DTC> = Vec::with_capacity(num_dtcs as usize); // Pre-allocate

    for x in (0..res.len()).step_by(3) {
        let status = res[x+2];
        ret.push(DTC {
            format: KWP_DTC_FMT,
            raw: (res[x] << 8) as u32 | res[x+1] as u32,
            status: DTCStatus::from_kwp_status(status),
            mil_on: status & 0b10000000 != 0,
            readiness_flag: status & 0b00010000 != 0
        })
    }
    Ok(ret)
}

/// Returns a list of all supported DTCs that the ECU supports, regardless of their status
pub fn read_supported_dtcs(server: &mut Kwp2000DiagnosticServer, range: DTCRange) -> DiagServerResult<Vec<DTC>> {
    let req = server.execute_command_with_response(KWP2000Command::ReadDiagnosticTroubleCodesByStatus, &range.to_args(0x01))?;
    todo!("ECU Response: {:02X?}", req)
}


/// Returns a list of all stored DTCs that the ECU has flagged,
/// regardless of their status, returning only their DTC code, without status
pub fn read_stored_dtcs_raw(server: &mut Kwp2000DiagnosticServer, range: DTCRange) -> DiagServerResult<Vec<u16>> {
    let req = server.execute_command_with_response(KWP2000Command::ReadDiagnosticTroubleCodesByStatus, &range.to_args(0x02))?;
    todo!("ECU Response: {:02X?}", req)
}

/// Returns a list of all supported DTCs that the ECU supports,
/// regardless of their status, returning only their DTC code, without status
pub fn read_supported_dtcs_raw(server: &mut Kwp2000DiagnosticServer, range: DTCRange) -> DiagServerResult<Vec<u16>> {
    let req = server.execute_command_with_response(KWP2000Command::ReadDiagnosticTroubleCodesByStatus, &range.to_args(0x03))?;
    todo!("ECU Response: {:02X?}", req)
    
}

/// Asks the ECU to report its most recent DTCs that has been stored.
/// Only one DTC is returned if stored, otherwise no DTC is returned.
pub fn get_most_recent_dtc(server: &mut Kwp2000DiagnosticServer, range: DTCRange) -> DiagServerResult<Option<DTC>> {
    let req = server.execute_command_with_response(KWP2000Command::ReadDiagnosticTroubleCodesByStatus, &range.to_args(0x04))?;
    todo!("ECU Response: {:02X?}", req)
}

/// Returns the total number of DTCs that the ECU supports, regardless
/// as to if they are active or not.
pub fn get_extended_number_of_supported_dtcs(server: &mut Kwp2000DiagnosticServer, range: DTCRange) -> DiagServerResult<u32> {
    let req = server.execute_command_with_response(KWP2000Command::ReadDiagnosticTroubleCodesByStatus, &range.to_args(0xE0))?;
    todo!("ECU Response: {:02X?}", req)
}