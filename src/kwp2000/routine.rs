//! Routine management wrapper for KWP2000

use crate::{dynamic_diag::DynamicDiagSession, DiagError, DiagServerResult};
use automotive_diag::kwp2000::{KwpCommand, KwpSessionType};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Routine Identifier
pub enum RoutineID {
    /// Routine local identifier (Range 0x01 - 0xDF)
    LocalIdentifier(u8),
    /// Flash erase routine
    FlashErase,
    /// Flash check routine
    FlashCheck,
    /// Request Diagnostic trouble codes from ECU shadow error memory
    RequestDTCFromShadowErrorMem,
    /// Request environmental data from shadow error memory
    RequestEnvDataFromShadowErrorMem,
    /// Request event information
    RequestEventInformation,
    /// Request Software module information
    RequestSWModuleInformation,
    /// Clear tell-tale retention stack
    ClearTellTaleRetentionStack,
    /// System supplier specific
    SystemSupplierSpecific(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Routine exit status
pub enum RoutineExitStatus {
    /// Unknown exit type
    Unknown(u8),
    /// Normal exit with results available. Call [KwpRoutineManager::request_routine_results] to obtain the results
    NormalExitWithResults,
    /// Normal exit, the routine does not return any result data
    NormalExitWithoutResults,
    /// Abnormal or premature exit. No results available.
    AbnormalExitWithoutResults,
}

impl From<u8> for RoutineExitStatus {
    fn from(x: u8) -> Self {
        match x {
            0x61 => Self::NormalExitWithResults,
            0x62 => Self::NormalExitWithoutResults,
            0x64 => Self::AbnormalExitWithoutResults,
            _ => Self::Unknown(x),
        }
    }
}

impl RoutineID {
    pub(crate) fn as_start_byte(&self) -> u8 {
        match self {
            RoutineID::LocalIdentifier(x) => *x,
            RoutineID::FlashErase => 0xE0,
            RoutineID::FlashCheck => 0xE1,
            RoutineID::RequestDTCFromShadowErrorMem => 0xE3,
            RoutineID::RequestEnvDataFromShadowErrorMem => 0xE4,
            RoutineID::RequestEventInformation => 0xE5,
            RoutineID::RequestSWModuleInformation => 0xE6,
            RoutineID::ClearTellTaleRetentionStack => 0xE7,
            RoutineID::SystemSupplierSpecific(x) => *x,
        }
    }

    pub(crate) fn as_result_byte(&self) -> u8 {
        match self {
            RoutineID::LocalIdentifier(x) => *x,
            RoutineID::FlashErase => 0xE0,
            RoutineID::FlashCheck => 0xE1,
            RoutineID::RequestDTCFromShadowErrorMem => 0xE3,
            RoutineID::RequestEnvDataFromShadowErrorMem => 0xE4,
            RoutineID::RequestEventInformation => 0xE5,
            RoutineID::RequestSWModuleInformation => 0xE6,
            RoutineID::ClearTellTaleRetentionStack => 0xE2,
            RoutineID::SystemSupplierSpecific(x) => *x,
        }
    }
}

#[derive(Debug)]
/// KWP2000 Routine execution wrapper
pub struct KwpRoutineManager<'a> {
    server: &'a mut DynamicDiagSession,
    r_id: RoutineID,
}

impl<'a> KwpRoutineManager<'a> {
    /// Creates a new routine manager. Upon creation, the KWP2000 diagnostic server will automatically
    /// attempt to enter extended diagnostic session mode, which is required for routine execution and
    /// management.
    ///
    /// # Parameters
    /// * rid - The routine ID
    /// * server - Reference to running KWP2000 diagnostic server
    ///
    /// # Returns
    /// If an error of [DiagError::ParameterInvalid] is returned, then it means that the value of `rid` is invalid
    /// and violates the KWP2000 specification. Other [DiagError]'s will come from the attempt to set the ECU
    /// into extended diagnostic session mode.
    pub fn new(rid: RoutineID, server: &'a mut DynamicDiagSession) -> DiagServerResult<Self> {
        let x: u8 = rid.as_start_byte();
        if x == 0x00 || x == 0xE2 || x == 0xFF || (0xEA..=0xF9).contains(&x) {
            return Err(DiagError::ParameterInvalid); // Unsupported by the spec, might have undefined behavior. Ignore!
        }
        // We have to be in extended mode for routine management to work!
        server.kwp_set_session(KwpSessionType::ExtendedDiagnostics.into())?;
        Ok(Self { server, r_id: rid })
    }

    /// Attempts to start the routine
    pub fn start_routine(&mut self, entry_options: &[u8]) -> DiagServerResult<()> {
        let mut p: Vec<u8> = vec![self.r_id.as_start_byte()];
        p.extend_from_slice(entry_options);
        self.server
            .send_command_with_response(KwpCommand::StartRoutineByLocalIdentifier, &p)?;
        Ok(())
    }

    /// Attempts to stop the routine. Note that some routines automatically exit themselves
    /// and do NOT need to be manually stopped
    pub fn stop_routine(&mut self, exit_options: &[u8]) -> DiagServerResult<RoutineExitStatus> {
        let mut p: Vec<u8> = vec![self.r_id.as_start_byte()];
        p.extend_from_slice(exit_options);
        self.server
            .send_command_with_response(KwpCommand::StopRoutineByLocalIdentifier, &p)
            .map(|x| x[1].into())
    }

    /// Requests the results of the routine. If the routine was manually stopped prior to running this,
    /// it is best practice to check the [RoutineExitStatus] to see if the routine exited with
    /// [RoutineExitStatus::NormalExitWithResults] first.
    pub fn request_routine_results(&mut self) -> DiagServerResult<Vec<u8>> {
        self.server.send_command_with_response(
            KwpCommand::RequestRoutineResultsByLocalIdentifier,
            &[self.r_id.as_result_byte()],
        )
    }
}
