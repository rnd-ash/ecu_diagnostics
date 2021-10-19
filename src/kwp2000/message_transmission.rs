//! Message transmission wrapper

use crate::{DiagServerResult, DiagnosticServer};

use super::{KWP2000Command, Kwp2000DiagnosticServer};


/// Tells the ECU to switch off its normal communication paths with other ECUs.
/// Normally message transmission will be resumed if the ECU looses power, or if
/// [enable_normal_message_transmission] is called.
/// 
/// NOTE: Calling this function can make a car go wild as it can no longer talk to
/// the ECU being sent this command. Use with CAUTION!
pub fn disable_normal_message_transmission(
    server: &mut Kwp2000DiagnosticServer,
) -> DiagServerResult<()> {
    server.execute_command_with_response(
        KWP2000Command::DisableNormalMessageTransmission,
        &[0x01],
    ).map(|_|())
}

/// Tells the ECU to re-enable its normal communication paths with other ECUs.
pub fn enable_normal_message_transmission(
    server: &mut Kwp2000DiagnosticServer,
) -> DiagServerResult<()> {
    server.execute_command_with_response(
        KWP2000Command::EnableNormalMessageTransmission,
        &[0x01],
    ).map(|_|())
}