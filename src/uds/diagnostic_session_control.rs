//!  Provides methods to manipulate the ECUs diagnostic session mode

use super::{UdsCommand, UdsDiagnosticServer};
use crate::{DiagServerResult, DiagnosticServer};

/// FIXME: This is deprecated, use UdsSessionType instead
/// Note: `#[deprecated]` doesn't work here due to https://github.com/rust-lang/rust/issues/30827
pub use auto_uds::UdsSessionType as UDSSessionType;

pub use auto_uds::UdsSessionType;

impl UdsDiagnosticServer {
    /// Requests the ECU to go into a specific diagnostic session mode
    pub fn set_session_mode(&mut self, session_mode: UdsSessionType) -> DiagServerResult<()> {
        self.execute_command_with_response(
            UdsCommand::DiagnosticSessionControl,
            &[session_mode.into()],
        )
        .map(|_| ())
    }
}
