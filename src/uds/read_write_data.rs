use crate::{DiagError, DiagServerResult, dynamic_diag::DynamicDiagSession};
use automotive_diag::uds::UdsCommand;

impl DynamicDiagSession {
    /// Reads data from the ECU by a 16-bit identifier (SID 0x22)
    ///
    /// ## Parameters
    /// * identifier - 16-bit DID to read
    ///
    /// ## Returns
    /// Raw data bytes from the ECU, with SID and DID stripped
    pub fn uds_read_data_by_identifier(&self, identifier: u16) -> DiagServerResult<Vec<u8>> {
        let mut resp = self.send_command_with_response(
            UdsCommand::ReadDataByIdentifier,
            &[(identifier >> 8) as u8, identifier as u8],
        )?;
        if resp.len() < 3 {
            return Err(DiagError::InvalidResponseLength);
        }
        resp.drain(0..3); // strip positive SID + DID bytes
        Ok(resp)
    }

    /// Writes data to the ECU by a 16-bit identifier (SID 0x2E)
    ///
    /// ## Parameters
    /// * identifier - 16-bit DID to write
    /// * data       - Data to write
    pub fn uds_write_data_by_identifier(
        &self,
        identifier: u16,
        data: &[u8],
    ) -> DiagServerResult<()> {
        let mut payload = vec![(identifier >> 8) as u8, identifier as u8];
        payload.extend_from_slice(data);
        self.send_command_with_response(UdsCommand::WriteDataByIdentifier, &payload)?;
        Ok(())
    }
}
