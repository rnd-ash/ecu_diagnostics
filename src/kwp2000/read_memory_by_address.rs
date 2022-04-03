//! Reads contents from the ECU's RAM

use crate::{DiagServerResult, DiagnosticServer};

use super::{KWP2000Command, Kwp2000DiagnosticServer};

impl Kwp2000DiagnosticServer {
    /// Reads the contents of RAM memory on the ECU given a 3 byte address, and 1 byte size.
    /// The maximum value for address is 0xFFFFFF, any larger values will be clamped.
    ///
    /// NOTE: This function is ONLY indented for ECU development. In production ECUs,
    /// use [Kwp2000DiagnosticServer::read_custom_local_identifier] instead
    pub fn read_memory(&mut self, address: u32, size: u8) -> DiagServerResult<Vec<u8>> {
        self.execute_command_with_response(
            KWP2000Command::ReadMemoryByAddress,
            &[
                (address >> 16) as u8,
                (address >> 8) as u8,
                address as u8,
                size,
            ],
        )
    }
}
