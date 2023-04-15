//! This service requests the ECU to perform a reset

use crate::{dynamic_diag::DynamicDiagSession, DiagServerResult};
use auto_uds::kwp2k::{KwpCommand, ResetType};

impl DynamicDiagSession {
    /// Performs an ECU Reset operation
    ///
    /// ## Params
    /// * mode - [ResetMode] to send to the ECU
    pub fn kwp_reset_ecu(&mut self, mode: ResetType) -> DiagServerResult<()> {
        self.send_command_with_response(KwpCommand::ECUReset, &[mode.into()])?;
        Ok(())
    }
}
