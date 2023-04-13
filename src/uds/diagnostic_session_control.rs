//!  Provides methods to manipulate the ECUs diagnostic session mode

use crate::{DiagServerResult, dynamic_diag::{DynamicDiagSession, DiagSessionMode}};

use auto_uds::UdsCommand;
pub use auto_uds::UdsSessionType as UDSSessionType;

impl DynamicDiagSession {
    /// Requests the ECU to go into a specific diagnostic session mode
    pub fn uds_set_session_mode(&mut self, session_mode: UDSSessionType) -> DiagServerResult<()> {
        self.send_command_with_response(
            UdsCommand::DiagnosticSessionControl,
            &[session_mode.into()],
        )
        .map(|_| ())
    }
}
