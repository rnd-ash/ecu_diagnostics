//! This service requests blocks of data from the ECU.

use crate::{DiagError, DiagServerResult, DiagnosticServer};

use super::{KWP2000Command, Kwp2000DiagnosticServer};

impl Kwp2000DiagnosticServer {
    /// Reads ECU data using a given identifier
    ///
    /// # Parameters
    /// * identifier - A 16 bit identifier to recall data from on the ECU
    ///
    /// ## Returns
    /// If successful, this function returns the raw data stored at this identifier,
    /// without the identifier ID on the response itself.
    pub fn read_data_by_identifier(&mut self, identifier: u16) -> DiagServerResult<Vec<u8>> {
        let mut res = self.execute_command_with_response(
            KWP2000Command::ReadDataByIdentifier,
            &[(identifier >> 8) as u8, identifier as u8],
        )?;
        // Now check identifier in response message was same as our request identifier, if so, strip it
        // from the response message
        if res.len() < 3 {
            // Require Positive SID, IDENT << 8, IDENT & 0xFF
            return Err(DiagError::InvalidResponseLength);
        }
        let ident_response = ((res[1] as u16) << 8) | (res[2] as u16);
        if ident_response != identifier {
            return Err(DiagError::MismatchedResponse(format!(
                "Expected identifier 0x{:04X}, got identifier 0x{:04X}",
                identifier, ident_response
            )));
        }
        res.drain(0..3);
        Ok(res)
    }
}
