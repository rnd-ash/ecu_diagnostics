//!  Provides methods to manipulate the ECUs diagnostic session mode

use crate::{dynamic_diag::DynamicDiagSession, DiagServerResult};

use automotive_diag::uds::{UdsCommand, UdsSessionTypeByte};

impl DynamicDiagSession {
    /// Requests the ECU to go into a specific diagnostic session mode
    pub fn uds_set_session_mode(&self, session_mode: UdsSessionTypeByte) -> DiagServerResult<()> {
        self.send_command_with_response(
            UdsCommand::DiagnosticSessionControl,
            &[session_mode.into()],
        )?;
        Ok(())
    }
}
