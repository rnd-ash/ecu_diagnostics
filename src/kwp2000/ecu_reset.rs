//! This service requests the ECU to perform a reset

use crate::DiagServerResult;

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
    Custom(u8)
}

impl From<ResetMode> for u8 {
    fn from(x: ResetMode) -> Self {
        match x {
            ResetMode::PowerOnReset => 0x01,
            ResetMode::NonVolatileMemoryReset => 0x82,
            ResetMode::Custom(x) => x
        }
    }
}

/// Performs an ECU Reset operation
/// 
/// ## Params
/// * server - KWP2000 diagnostic server
/// * mode - [ResetMode] to send to the ECU
pub fn execute_reset(server: &mut Kwp2000DiagnosticServer, mode: ResetMode) -> DiagServerResult<()> {
    server.execute_command_with_response(KWP2000Command::ECUReset, &[mode.into()]).map(|_| ())
}