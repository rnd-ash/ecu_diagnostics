//! Provides methods to control normal ECU communication

use crate::uds::{UdsCommand, UdsDiagnosticServer};
use crate::DiagnosticServer;

pub use auto_uds::{
    encode_communication_type, CommunicationLevel, CommunicationType as EcuCommunicationType,
    Subnet,
};

impl UdsDiagnosticServer {
    /// Modifies ECU communication settings. These settings persist until the ECU is power cycled
    ///
    /// ## Parameters
    /// * server - The UDS diagnostic server
    /// * communication_type - Communication layer to modify
    /// * Subnet - The subnet the ECU communicates with to modify
    /// * comm_level - Communication level
    pub fn control_communication(
        &mut self,
        communication_type: EcuCommunicationType,
        subnet: Subnet,
        comm_level: CommunicationLevel,
    ) -> super::DiagServerResult<()> {
        let level: u8 = comm_level.into();
        let communication_type = encode_communication_type(communication_type, subnet);

        self.execute_command_with_response(
            UdsCommand::CommunicationControl,
            &[level, communication_type],
        )
        .map(|_| ())
    }
}
