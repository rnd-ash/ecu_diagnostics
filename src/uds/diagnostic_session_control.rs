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

impl UdsDiagnosticServer {
    /// Requests the ECU to go into a specific diagnostic session mode
    pub fn set_session_mode(&mut self, session_mode: UDSSessionType) -> DiagServerResult<()> {
        self.execute_command_with_response(
            UDSCommand::DiagnosticSessionControl,
            &[session_mode.into()],
        )
        .map(|_| ())
    }
}
