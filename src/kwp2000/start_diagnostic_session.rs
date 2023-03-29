//! Provides methods to manipulate the ECUs diagnostic session mode

use crate::{DiagServerResult, DiagnosticServer, dynamic_diag::{DiagSessionMode, DynamicDiagSession}};

use super::{KWP2000Command, Kwp2000DiagnosticServer, Kwp2000Protocol};

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
pub enum KwpSessionType {
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
    Custom { id: u8, name: &'static str, tp_required: bool },
}

impl From<KwpSessionType> for DiagSessionMode {
    fn from(x: KwpSessionType) -> Self {
        match x {
            KwpSessionType::Normal => 0x81,
            KwpSessionType::Reprogramming => 0x85,
            KwpSessionType::Standby => 0x89,
            KwpSessionType::Passive => 0x90,
            KwpSessionType::ExtendedDiagnostics => 0x92,
            KwpSessionType::Custom(c) => c,
        }
    }
}

impl DynamicDiagSession<Kwp2000Protocol> {
    /// Sets the ECU into a diagnostic mode
    ///
    /// ## Parameters
    /// * server - The KWP2000 Diagnostic server
    /// * mode - The [KwpSessionType] to put the ECU into
    
    pub fn set_diag_session_mode(&self, mode: Kwp) {

    }

    pub fn set_diagnostic_session_mode(&mut self, mode: KwpSessionType) -> DiagServerResult<()> {
        self.execute_command_with_response(KWP2000Command::StartDiagnosticSession, &[mode.into()])
            .map(|_| ())
    }
}
