//! Provides methods to manipulate the ECUs diagnostic session mode

use crate::{DiagServerResult, DiagnosticServer};

use super::{KWP2000Command, Kwp2000DiagnosticServer};

/// KWP2000 diagnostic session type
///
/// Session support matrix
///
/// | SessionType | Support by ECUs |
/// |--|--|
/// |[SessionType::Normal] | Mandatory |
/// |[SessionType::Reprogramming] | Optional (Only ECUs which implement the ECU-Flash reprogramming specification) |
/// |[SessionType::Standby] | Optional |
/// |[SessionType::Passive] | Optional (Only intended for ECU development) |
/// |[SessionType::ExtendedDiagnostics] | Mandatory |
/// |[SessionType::Custom] | Optional |
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SessionType {
    /// Normal session. The ECU will typically boot in this state.
    /// In this mode, only non-intrusive functions are supported.
    Normal,
    /// Reprogramming session. Used for flashing an ECU. Only functions
    /// for reading/writing to memory are allowed in this mode
    Reprogramming,
    /// In stand-by mode, the ECU will be in a low-power state,
    /// acting as a slave to other ECUs and only able to perform actuation tests
    /// at the request of a tester. If a request is made to the ECU which can disrupt
    /// its low power state, the ECU will reject the request.
    Standby,
    /// In this mode, the ECU will remain active, but will disable
    /// all normal CAN/LIN communication with the vehicle, effectively putting
    /// the ECU to sleep. IMPORTANT. If the ECU is power cycled, it will
    /// reboot in this mode.
    Passive,
    /// Extended diagnostics mode. Every service is available here
    ExtendedDiagnostics,
    /// Custom diagnostic mode not in the KWP2000 specification
    Custom(u8),
}

impl From<SessionType> for u8 {
    fn from(x: SessionType) -> Self {
        match x {
            SessionType::Normal => 0x81,
            SessionType::Reprogramming => 0x85,
            SessionType::Standby => 0x89,
            SessionType::Passive => 0x90,
            SessionType::ExtendedDiagnostics => 0x92,
            SessionType::Custom(c) => c,
        }
    }
}

/// Sets the ECU into a diagnostic mode
///
/// ## Parameters
/// * server - The KWP2000 Diagnostic server
/// * mode - The [SessionType] to put the ECU into
pub fn set_diagnostic_session_mode(
    server: &mut Kwp2000DiagnosticServer,
    mode: SessionType,
) -> DiagServerResult<()> {
    server
        .execute_command_with_response(KWP2000Command::StartDiagnosticSession, &[mode.into()])
        .map(|_| ())
}
