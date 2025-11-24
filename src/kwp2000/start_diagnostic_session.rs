//! Provides methods to manipulate the ECUs diagnostic session mode

use crate::{dynamic_diag::DynamicDiagSession, DiagServerResult};
use automotive_diag::kwp2000::{KwpCommand, KwpSessionTypeByte};

impl DynamicDiagSession {
    /// Set KWP session mode
    pub fn kwp_set_session(&self, mode: KwpSessionTypeByte) -> DiagServerResult<()> {
        self.send_command_with_response(KwpCommand::StartDiagnosticSession  , &[mode.into()])?;
        Ok(())
    }
}
