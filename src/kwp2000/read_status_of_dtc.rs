//! Reads environmental data from the ECU about a requested Diagnostic
//! trouble code (DTC).

use crate::{dynamic_diag::DynamicDiagSession, DiagServerResult};
use auto_uds::kwp2k::KwpCommand;

impl DynamicDiagSession {
    /// Reads the status of a given DTC.
    ///
    /// This function returns bytes rather than a processed result as the environmental data
    /// varies from DTC to DTC and from ECU to ECU, so it is impossible to know what the data
    /// returned actually means.
    ///
    /// ## Returns
    /// This function if successful will return the full ECUs response message without
    /// any additional processing.
    ///
    /// The first 4 bytes of the response are as follows:
    /// 1. DTC Number (Stored on ECU)
    /// 2. DTC High byte
    /// 3. DTC Low byte
    /// 4. Status of DTC
    pub fn kwp_read_status_of_dtc(&mut self, dtc: u16) -> DiagServerResult<Vec<u8>> {
        self.send_command_with_response(
            KwpCommand::ReadStatusOfDiagnosticTroubleCodes,
            &[(dtc >> 8) as u8, dtc as u8],
        )
    }
}
