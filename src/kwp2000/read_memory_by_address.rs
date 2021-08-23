//! Reads contents from the ECU's RAM

use crate::{DiagServerResult, DiagnosticServer};

use super::{KWP2000Command, Kwp2000DiagnosticServer};

/// Reads the contents of RAM memory on the ECU given a 3 byte address, and 1 byte size.
/// The maximum value for address is 0xFFFFFF, any larger values will be clamped.
///
/// NOTE: This function is ONLY indented for ECU development. In production ECUs,
/// use [super::read_data_by_local_id] instead
pub fn read_memory(
    server: &mut Kwp2000DiagnosticServer,
    address: u32,
    size: u8,
) -> DiagServerResult<Vec<u8>> {
    server.execute_command_with_response(
        KWP2000Command::ReadMemoryByAddress,
        &[
            (address >> 16) as u8,
            (address >> 8) as u8,
            address as u8,
            size,
        ],
    )
}
