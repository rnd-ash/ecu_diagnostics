//! Provides methods to manipulate the ECUs diagnostic session mode

use crate::{dynamic_diag::DynamicDiagSession, DiagServerResult};
use automotive_diag::kwp2000::KwpSessionTypeByte;

impl DynamicDiagSession {
    /// Set KWP session mode
    pub fn kwp_set_session(&self, mode: KwpSessionTypeByte) -> DiagServerResult<()> {
        self.send_byte_array_with_response(&[0x10, mode.into()])?;
        Ok(())
    }
}
