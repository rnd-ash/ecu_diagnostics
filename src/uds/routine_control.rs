use crate::{DiagServerResult, dynamic_diag::DynamicDiagSession};
use automotive_diag::uds::{RoutineControlType, UdsCommand};

impl DynamicDiagSession {
    /// Executes a routine control request (SID 0x31)
    ///
    /// ## Parameters
    /// * control_type - Start, Stop or RequestResults
    /// * routine_id   - 16-bit routine identifier
    /// * params       - Optional extra parameters (can be empty)
    pub fn uds_routine_control(
        &self,
        control_type: RoutineControlType,
        routine_id: u16,
        params: &[u8],
    ) -> DiagServerResult<Vec<u8>> {
        let mut payload = vec![
            control_type as u8,
            (routine_id >> 8) as u8,
            routine_id as u8,
        ];
        payload.extend_from_slice(params);
        self.send_command_with_response(UdsCommand::RoutineControl, &payload)
    }
}
