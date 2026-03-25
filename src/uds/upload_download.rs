use crate::{DiagServerResult, dynamic_diag::DynamicDiagSession};
use automotive_diag::uds::UdsCommand;

impl DynamicDiagSession {
    /// Transfers a block of data to the ECU (SID 0x36)
    ///
    /// ## Parameters
    /// * block - Block sequence counter (starts at 0x00, wraps at 0xFF)
    /// * data - Data payload for this block
    pub fn uds_transfer_data(&self, block: u8, data: &[u8]) -> DiagServerResult<Vec<u8>> {
        let mut payload = vec![block];
        payload.extend_from_slice(data);
        self.send_command_with_response(UdsCommand::TransferData, &payload)
    }

    /// Ends the data transfer sequence (SID 0x37)
    pub fn uds_request_transfer_exit(&self) -> DiagServerResult<Vec<u8>> {
        self.send_command_with_response(UdsCommand::RequestTransferExit, &[])
    }
}
