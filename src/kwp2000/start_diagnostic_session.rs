//! Provides methods to manipulate the ECUs diagnostic session mode

use crate::{dynamic_diag::{DiagSessionMode, DynamicDiagSession}, DiagServerResult};


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
    Custom { id: u8 },
}

impl From<KwpSessionType> for DiagSessionMode {
    fn from(x: KwpSessionType) -> Self {
        match x {
            KwpSessionType::Normal => DiagSessionMode {
                id: 0x81,
                tp_require: false,
                name: "Normal",
            },
            KwpSessionType::Reprogramming => DiagSessionMode {
                id: 0x85,
                tp_require: true,
                name: "Reprogramming",
            },
            KwpSessionType::Standby => DiagSessionMode {
                id: 0x89,
                tp_require: true,
                name: "Standby",
            },
            KwpSessionType::Passive => DiagSessionMode {
                id: 0x90,
                tp_require: false,
                name: "Passive",
            },
            KwpSessionType::ExtendedDiagnostics => DiagSessionMode {
                id: 0x92,
                tp_require: true,
                name: "ExtendedDiagnostics",
            },
            KwpSessionType::Custom { id: c} => DiagSessionMode {
                id: c,
                tp_require: true,
                name: "Custom",
            },
        }
    }
}

impl From<u8> for KwpSessionType {
    fn from(value: u8) -> Self {
        match value {
            0x81 => Self::Normal,
            0x85 => Self::Reprogramming,
            0x89 => Self::Standby,
            0x90 => Self::Passive,
            0x92 => Self::ExtendedDiagnostics,
            x => Self::Custom { id: x }
        }
    }
}

impl Into<u8> for KwpSessionType {
    fn into(self) -> u8 {
        match self {
            KwpSessionType::Normal => 0x81,
            KwpSessionType::Reprogramming => 0x85,
            KwpSessionType::Standby => 0x89,
            KwpSessionType::Passive => 0x90,
            KwpSessionType::ExtendedDiagnostics => 0x92,
            KwpSessionType::Custom { id } => id,
        }
    }
}

impl DynamicDiagSession {
    pub fn kwp_set_session(&mut self, mode: KwpSessionType) -> DiagServerResult<()> {
        self.send_byte_array_with_response(&[0x10, mode.into()]).map(|_|())
    }
}
