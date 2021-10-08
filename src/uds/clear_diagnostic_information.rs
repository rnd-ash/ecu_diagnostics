//!  Provides methods to clear diagnostic trouble codes from the ECU

use super::{UDSCommand, UdsDiagnosticServer};
use crate::{DiagServerResult, DiagnosticServer};

/// Clears diagnostic information (DTCs) from the ECU.
/// 
/// ## Parameters
/// * server - The UDS Diagnostic server
/// * dtc_mask - Mask of DTCs to clear. Only the lower 3 bytes are used (from 0x00000000 - 0x00FFFFFF)
pub fn clear_diagnostic_information(server: &mut UdsDiagnosticServer, dtc_mask: u32) -> DiagServerResult<()> {
    server
        .execute_command_with_response(UDSCommand::ClearDiagnosticInformation, &[(dtc_mask >> 16) as u8, (dtc_mask >> 8) as u8, dtc_mask as u8])
        .map(|_| ())
}