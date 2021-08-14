//! This service requests blocks of data from the ECU.

use crate::DiagServerResult;

use super::{KWP2000Command, Kwp2000DiagnosticServer};

/// Reads ECU data using a given identifier
pub fn read_data(
    server: &mut Kwp2000DiagnosticServer,
    identifier: u16,
) -> DiagServerResult<Vec<u8>> {
    server.execute_command_with_response(
        KWP2000Command::ReadDataByIdentifier,
        &[(identifier >> 8) as u8, identifier as u8],
    )
}
