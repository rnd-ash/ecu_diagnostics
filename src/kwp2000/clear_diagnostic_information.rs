//! This service allows for the clearing of DTCs
//! (Diagnostic trouble codes) from the ECU

use crate::{DiagServerResult, DiagnosticServer};

use super::{KWP2000Command, Kwp2000DiagnosticServer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Denotes a single or range of DTCs that can be cleared from the ECU
///
/// Command support matrix
///
/// | DTCRange | Support by ECUs |
/// |--|--|
/// |[ClearDTCRange::AllPowertrain]|Optional|
/// |[ClearDTCRange::AllChassis]|Optional|
/// |[ClearDTCRange::AllBody]|Optional|
/// |[ClearDTCRange::AllNetwork]|Optional|
/// |[ClearDTCRange::AllDTCs]|Mandatory|
/// |[ClearDTCRange::SingleDTC]|Optional|
pub enum ClearDTCRange {
    /// All powertrain related DTCs
    AllPowertrain,
    /// All Chassis related DTCs
    AllChassis,
    /// All body related DTCs
    AllBody,
    /// All network related DTCs
    AllNetwork,
    /// All DTCs stored on the ECU
    AllDTCs,
    /// Denotes a single specific DTC to clear.
    ///
    /// Acceptable ranges:
    /// * 0x0001-0x3FFF - Custom powertrain DTC
    /// * 0x4001-0x7FFF - Custom chassis DTC
    /// * 0x8001-0xBFFF - Custom body DTC
    /// * 0xC001-0xFEFF - Custom network DTC
    SingleDTC(u16),
}

impl From<ClearDTCRange> for u16 {
    fn from(x: ClearDTCRange) -> Self {
        match x {
            ClearDTCRange::AllPowertrain => 0x0000,
            ClearDTCRange::AllChassis => 0x4000,
            ClearDTCRange::AllBody => 0x8000,
            ClearDTCRange::AllNetwork => 0xC000,
            ClearDTCRange::AllDTCs => 0xFF00,
            ClearDTCRange::SingleDTC(x) => x,
        }
    }
}

/// Executes a DTC clear command on the ECU, given a range of DTCs to clear
pub fn clear_dtc(
    server: &mut Kwp2000DiagnosticServer,
    dtc_range: ClearDTCRange,
) -> DiagServerResult<()> {
    let dtc_range_num: u16 = dtc_range.into();
    server
        .execute_command_with_response(
            KWP2000Command::ClearDiagnosticInformation,
            &[(dtc_range_num >> 8) as u8, dtc_range_num as u8],
        )
        .map(|_| ())
}
