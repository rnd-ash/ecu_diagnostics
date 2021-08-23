//!  Provides methods to manipulate the ECUs diagnostic session mode

use crate::{DiagServerResult, DiagnosticServer};

use super::{UDSCommand, UdsDiagnosticServer};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// UDS Diagnostic session modes. Handled by SID 0x10
pub enum UDSSessionType {
    /// Default diagnostic session mode (ECU is normally in this mode on startup)
    /// This session type does not require the diagnostic server to sent TesterPresent messages
    Default,

    /// This diagnostic session mode enables all diagnostic services related to flashing or programming
    /// the ECU
    Programming,

    /// This diagnostic session mode enabled all diagnostic services and allows adjusting
    /// ECU values
    Extended,

    /// This diagnostic session enables all diagnostic services required to support safety system-related functions
    SafetySystem,

    /// Custom session type. This covers both vehicleManufacturerSpecific modes (0x40-0x5F) and systemSupplierSpecific modes (0x60-0x7E).
    Other(u8),
}

impl From<UDSSessionType> for u8 {
    fn from(from: UDSSessionType) -> u8 {
        match &from {
            UDSSessionType::Default => 0x01,
            UDSSessionType::Programming => 0x02,
            UDSSessionType::Extended => 0x03,
            UDSSessionType::SafetySystem => 0x04,
            &UDSSessionType::Other(x) => x,
        }
    }
}

/// Tells the ECU to enter default diagnostic session mode
///
/// ## Parameters
/// * server - The UDS Diagnostic server
pub fn set_default_mode(server: &mut UdsDiagnosticServer) -> DiagServerResult<()> {
    server
        .execute_command_with_response(
            UDSCommand::DiagnosticSessionControl,
            &[UDSSessionType::Default.into()],
        )
        .map(|_| ())
}

/// Tells the ECU to enter a programming diagnostic session mode
///
/// ## Parameters
/// * server - The UDS Diagnostic server
pub fn set_programming_mode(server: &mut UdsDiagnosticServer) -> DiagServerResult<()> {
    server
        .execute_command_with_response(
            UDSCommand::DiagnosticSessionControl,
            &[UDSSessionType::Programming.into()],
        )
        .map(|_| ())
}

/// Tells the ECU to enter an extended diagnostic session mode
///
/// ## Parameters
/// * server - The UDS Diagnostic server
pub fn set_extended_mode(server: &mut UdsDiagnosticServer) -> DiagServerResult<()> {
    server
        .execute_command_with_response(
            UDSCommand::DiagnosticSessionControl,
            &[UDSSessionType::Extended.into()],
        )
        .map(|_| ())
}

/// Tells the ECU to enter a safety system diagnostic session mode
///
/// ## Parameters
/// * server - The UDS Diagnostic server
pub fn set_safety_system_mode(server: &mut UdsDiagnosticServer) -> DiagServerResult<()> {
    server
        .execute_command_with_response(
            UDSCommand::DiagnosticSessionControl,
            &[UDSSessionType::SafetySystem.into()],
        )
        .map(|_| ())
}

/// Tells the ECU to enter a custom diagnostic session mode
///
/// ## Parameters
/// * server - The UDS Diagnostic server
/// * custom_mode_id - Custom diagnostic session mode
pub fn set_custom_mode(
    server: &mut UdsDiagnosticServer,
    custom_mode_id: u8,
) -> DiagServerResult<()> {
    server
        .execute_command_with_response(
            UDSCommand::DiagnosticSessionControl,
            &[UDSSessionType::Other(custom_mode_id).into()],
        )
        .map(|_| ())
}
