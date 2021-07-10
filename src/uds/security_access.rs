//! Provides methods for security seed/key access to the ECU in order to unlock functions which
//! are considered secure such as writing or reading to specific memory regions on the ECU
//!
//! Currently, only default seed/key (0x01/0x02) are supported

use super::{UDSCommand, UdsDiagnosticServer};
use crate::DiagServerResult;

/// Security operation request
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityOperation {
    /// Asks the ECU for a security seed
    RequestSeed,
    /// Sends the computed key to the ECU
    SendKey,
}

impl From<SecurityOperation> for u8 {
    fn from(from: SecurityOperation) -> Self {
        match from {
            SecurityOperation::RequestSeed => 0x01,
            SecurityOperation::SendKey => 0x02,
        }
    }
}

/// Requests a seed from the ECU for security access.
///
/// Once the key is calculated from the response seed, run [send_key] to send the computed key to the ECU
///
/// ## Parameters
/// * server - The UDS Diagnostic server
///
/// ## Returns
/// Returns the security key's seed
pub fn request_seed(server: &mut UdsDiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let mut resp = server.execute_command_with_response(
        UDSCommand::SecurityAccess,
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
pub fn send_key(server: &mut UdsDiagnosticServer, key: &[u8]) -> DiagServerResult<()> {
    let mut payload = Vec::with_capacity(key.len() + 1);
    payload.push(SecurityOperation::SendKey.into());
    payload.extend_from_slice(key);
    server
        .execute_command_with_response(UDSCommand::SecurityAccess, &payload)
        .map(|_| ())
}
