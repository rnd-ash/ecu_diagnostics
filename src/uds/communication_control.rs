//! Provides methods to control normal ECU communication

use crate::DiagnosticServer;
use crate::uds::{UDSCommand, UdsDiagnosticServer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Communication level toggle
pub enum CommunicationLevel {
    /// Enable both Rx and Tx communication
    EnableRxAndTx,
    /// Enable Rx communication and disable Tx communication
    EnableRxDisableTx,
    /// Disable Rx communication and enable Tx communication
    DisableRxEnableTx,
    /// Disable both Rx and Tx communication
    DisableRxAndTx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// ECU Communication types
pub enum EcuCommunicationType {
    /// Application layer communication (inter-signal exchanges)
    /// between ECUs
    NormalCommunication,
    /// Network management related communication
    NetworkManagement,
    /// Both application layer communication and network management communication
    All
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// ECU communication subnet type
pub enum Subnet {
    /// All subnets
    All,
    /// Custom Subnet ID. Values range from 0x01-0x0E
    Custom(u8),
    /// Only received subnets
    RxOnly
}

/// Modifies ECU communication settings. These settings persist until the ECU is power cycled
///
/// ## Parameters
/// * server - The UDS diagnostic server
/// * communication_type - Communication layer to modify
/// * Subnet - The subnet the ECU communicates with to modify
/// * comm_level - Communication level
pub fn control_communication(server: &mut UdsDiagnosticServer,  communication_type: EcuCommunicationType, subnet: Subnet, comm_level: CommunicationLevel) -> super::DiagServerResult<()> {
    // Encode communication_Type
    let mut communication_type: u8 = match communication_type {
        EcuCommunicationType::NormalCommunication => 0x01,
        EcuCommunicationType::NetworkManagement => 0x02,
        EcuCommunicationType::All => 0x03,
    };
    communication_type |= (match subnet {
        Subnet::All => 0x00,
        Subnet::Custom(x) => x << 4,
        Subnet::RxOnly => 0x0F
    }) << 4;

    let level: u8 = match comm_level {
        CommunicationLevel::EnableRxAndTx => 0x00,
        CommunicationLevel::EnableRxDisableTx => 0x01,
        CommunicationLevel::DisableRxEnableTx => 0x02,
        CommunicationLevel::DisableRxAndTx => 0x03
    };

    server.execute_command_with_response(UDSCommand::CommunicationControl, &[level, communication_type])
        .map(|_| ())
}