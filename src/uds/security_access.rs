//! Provides methods for security seed/key access to the ECU in order to unlock functions which
//! are considered secure such as writing or reading to specific memory regions on the ECU
//!
//! Currently, only default seed/key (0x01/0x02) are supported
//! 
use crate::{DiagServerResult, dynamic_diag::DynamicDiagSession};

pub use auto_uds::SecurityOperation;

impl DynamicDiagSession {
    /// Requests a seed from the ECU for security access.
    ///
    /// Once the key is calculated from the response seed, run [UdsDiagnosticServer::send_key] to send the computed key to the ECU
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    ///
    /// ## Returns
    /// Returns the security key's seed
    pub fn uds_request_seed(&mut self) -> DiagServerResult<Vec<u8>> {
        let mut resp = self.send_command_with_response(
            auto_uds::Command::SecurityAccess,
            &[SecurityOperation::RequestSeed.into()],
        )?;
        resp.drain(0..2); // Remove SID and PID, so just seed value left
        Ok(resp)
    }

    /// Sends the computed key to the ECU.
    ///
    /// If this function is successful, then the ECU has now allows access to security protected memory regions and functions
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    /// * key - The computed key to send to the ECU
    pub fn uds_send_key(&mut self, key: &[u8]) -> DiagServerResult<()> {
        let mut payload = Vec::with_capacity(key.len() + 1);
        payload.push(SecurityOperation::SendKey.into());
        payload.extend_from_slice(key);
        self.send_command_with_response(auto_uds::Command::SecurityAccess, &payload)
            .map(|_| ())
    }
}
