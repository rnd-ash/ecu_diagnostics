//! Functions for unlocking secure regions on the ECU

use super::{KWP2000Command};
use crate::{DiagError, DiagServerResult, DiagnosticServer, dynamic_diag::DynamicDiagSession};

impl DynamicDiagSession {
    /// Requests a seed from the ECU
    ///
    /// ## Parameters
    /// * server - The KWP2000 server
    /// * access_mode - The access mode. Only odd numbers between 0x01-0x7F are supported for the access level!
    ///
    /// ## Returns
    /// This function will return an error if the access_mode parameter is not a valid mode!
    /// If the function succeeds, then just the ECUs key response is returned
    pub fn kwp_request_seed(&mut self, access_mode: u8) -> DiagServerResult<Vec<u8>> {
        if access_mode % 2 == 0 {
            return Err(DiagError::ParameterInvalid);
        }
        let mut res =
            self.send_command_with_response(KWP2000Command::SecurityAccess, &[access_mode])?;
        res.drain(0..2); // Remove SID and access mode
        Ok(res) // Return just the key
    }

    /// Attempts to unlock the access mode to the ECU, using a computed key using the seed provided with [Kwp2000DiagnosticServer::request_seed]
    ///
    /// ## Parameters
    /// * server - The KWP2000 server
    /// * access_mode - The access mode. Only odd numbers between 0x01-0x7F are supported for the access level!
    ///
    /// ## Returns
    /// This function will return an error if the access_mode parameter is not a valid mode! The access_mode
    /// should be THE SAME as what was provided to [Kwp2000DiagnosticServer::request_seed]
    pub fn kwp_unlock_ecu_with_key(&mut self, access_mode: u8, key: &[u8]) -> DiagServerResult<()> {
        if access_mode % 2 == 0 {
            return Err(DiagError::ParameterInvalid);
        }
        let mut args = vec![access_mode + 1];
        args.extend_from_slice(key);
        self.send_command_with_response(KWP2000Command::SecurityAccess, &args)
            .map(|_| ())
    }
}
