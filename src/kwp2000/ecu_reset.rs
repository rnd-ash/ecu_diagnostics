//! This service requests the ECU to perform a reset

use crate::{DiagServerResult, DiagnosticServer};

use super::{KWP2000Command, Kwp2000DiagnosticServer};

/// ECU Reset types
///
/// Command support matrix
///
/// | ResetMode | Support by ECUs |
/// |--|--|
/// |[ResetMode::PowerOnReset]|Mandatory|
/// |[ResetMode::NonVolatileMemoryReset]|Optional|
/// |[ResetMode::Custom]|Optional|
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResetMode {
    /// Simulates a power off/on reset of the ECU.
    PowerOnReset,
    /// Just resets Non volatile memory of the ECU, resetting it
    NonVolatileMemoryReset,
    /// Custom reset mode (Not defined by KWP2000 specification)
    Custom(u8),
}

impl From<ResetMode> for u8 {
    fn from(x: ResetMode) -> Self {
        match x {
            ResetMode::PowerOnReset => 0x01,
            ResetMode::NonVolatileMemoryReset => 0x82,
            ResetMode::Custom(x) => x,
        }
    }
}

impl Kwp2000DiagnosticServer {
    /// Performs an ECU Reset operation
    ///
    /// ## Params
    /// * mode - [ResetMode] to send to the ECU
    pub fn reset_ecu(&mut self, mode: ResetMode) -> DiagServerResult<()> {
        self.execute_command_with_response(KWP2000Command::ECUReset, &[mode.into()])
            .map(|_| ())
    }
}
