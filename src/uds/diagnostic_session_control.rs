//!  Provides methods to manipulate the ECUs diagnostic session mode

use crate::{DiagServerResult, DiagnosticServer};

use super::{UDSCommand, UdsDiagnosticServer};

pub use auto_uds::UdsSessionType as UDSSessionType;

impl UdsDiagnosticServer {
    /// Requests the ECU to go into a specific diagnostic session mode
    pub fn set_session_mode(&mut self, session_mode: UDSSessionType) -> DiagServerResult<()> {
        self.execute_command_with_response(
            UDSCommand::DiagnosticSessionControl,
            &[session_mode.into()],
        )
        .map(|_| ())
    }
}
