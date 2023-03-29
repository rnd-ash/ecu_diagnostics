//!  Provides methods to reset the ECU in order to simulate power cycling and resetting memory regions

use super::{lookup_uds_nrc, UdsDiagnosticServer};
use crate::{DiagError, DiagServerResult, DiagnosticServer};
use auto_uds::UdsCommand::*;

pub use auto_uds::ResetType;

impl UdsDiagnosticServer {
    /// Asks the ECU to perform a hard reset. See [ResetType::HardReset] for more details
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    pub fn ecu_hard_reset(&mut self) -> DiagServerResult<()> {
        self.execute_command_with_response(ECUReset, &[ResetType::HardReset.into()])
            .map(|_| ())
    }

    /// Asks the ECU to perform a key off/on reset. See [ResetType::KeyOffReset] for more details
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    pub fn ecu_key_off_on_reset(&mut self) -> DiagServerResult<()> {
        self.execute_command_with_response(ECUReset, &[ResetType::KeyOffReset.into()])
            .map(|_| ())
    }

    /// Asks the ECU to perform a soft reset. See [ResetType::SoftReset] for more details
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    pub fn ecu_soft_reset(&mut self) -> DiagServerResult<()> {
        self.execute_command_with_response(ECUReset, &[ResetType::SoftReset.into()])
            .map(|_| ())
    }

    /// Asks the ECU to enable rapid power shutdown mode. See [ResetType::EnableRapidPowerShutDown] for more details
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    ///
    /// ## Returns
    /// If successful, this function will return the minimum time in seconds that the ECU will remain in the power-down sequence
    pub fn enable_rapid_power_shutdown(&mut self) -> DiagServerResult<u8> {
        let res = self.execute_command_with_response(
            ECUReset,
            &[ResetType::EnableRapidPowerShutDown.into()],
        )?;
        match res.get(2) {
            Some(&time) if time == 0xFF => Err(DiagError::ECUError {
                code: 0x10,
                def: Some(lookup_uds_nrc(0x10)),
            }), // General reject
            Some(&time) => Ok(time),
            None => Err(DiagError::InvalidResponseLength),
        }
    }

    /// Asks the ECU to disable rapid power shutdown mode
    ///
    /// ## Parameters
    /// * server - The UDS Diagnostic server
    pub fn disable_rapid_power_shutdown(&mut self) -> DiagServerResult<()> {
        self.execute_command_with_response(ECUReset, &[ResetType::DisableRapidPowerShutDown.into()])
            .map(|_| ())
    }
}
