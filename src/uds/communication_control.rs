//! Provides methods to control normal ECU communication

use automotive_diag::uds::UdsCommand;
pub use automotive_diag::uds::{
    encode_communication_type, CommunicationLevel, CommunicationType as EcuCommunicationType,
    Subnet,
};

use crate::{dynamic_diag::DynamicDiagSession, DiagServerResult};

impl DynamicDiagSession {
    /// Modifies ECU communication settings. These settings persist until the ECU is power cycled
    ///
    /// ## Parameters
    /// * server - The UDS diagnostic server
    /// * communication_type - Communication layer to modify
    /// * Subnet - The subnet the ECU communicates with to modify
    /// * comm_level - Communication level
    pub fn uds_control_communication(
        &mut self,
        communication_type: EcuCommunicationType,
        subnet: Subnet,
        comm_level: CommunicationLevel,
    ) -> DiagServerResult<()> {
        let level: u8 = comm_level.into();
        let communication_type = encode_communication_type(communication_type, subnet);

        self.send_command_with_response(
            UdsCommand::CommunicationControl,
            &[level, communication_type],
        )?;
        Ok(())
    }
}
