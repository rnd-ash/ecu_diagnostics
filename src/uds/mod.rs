use core::marker::PhantomData;

use crate::{AdvancedECUDiagServer, BasicECUDiagServer, DiagError, ECUCommChannel};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UDSSessionType {
    Default,
    Programming,
    Extended,
    SafetySystem,
    Other(u8)
}

impl Into<u8> for UDSSessionType {
    fn into(self) -> u8 {
        match &self {
            UDSSessionType::Default => 0x01,
            UDSSessionType::Programming => 0x02,
            UDSSessionType::Extended => 0x03,
            UDSSessionType::SafetySystem => 0x04,
            UDSSessionType::Other(x) => *x,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UDSError {
    GeneralReject,
    ServiceNotSupported,
    SubFunctionNotSupported,
    IncorrectMessageLengthOrInvalidFormat,
    ResponseTooLong,
    BusyRepeatRequest,
    ConditionsNotCorrect,
    RequestSequenceError,
    NoResponseFromSubnetComponent,
    FailurePreventsExecutionOfRequestedAction,
    RequestOutOfRange,
    SecurityAccessDenied,
    InvalidKey,
    ExceedNumberOfAttempts,
    RequiredTimeDelayNotExpired,
    UploadDownloadNotAccepted,
    TransferDataSuspended,
    GeneralProgrammingFailure,
    WrongBlockSequenceCounter,
    RequestCorrectlyReceivedResponsePending,
    SubFunctionNotSupportedInActiveSession,
    ServiceNotSupportedInActiveSession,
    RpmTooHigh,
    RpmTooLow,
    EngineIsRunning,
    EngineIsNotRunning,
    EngineRunTimeTooLow,
    TemperatureTooHigh,
    TemperatureTooLow,
    VehicleSpeedTooHigh,
    VehicleSpeedTooLow,
    ThrottleTooHigh,
    ThrottleTooLow,
    TransmissionRangeNotInNeutral,
    TransmissionRangeNotInGear,
    BrakeSwitchNotClosed,
    ShifterLeverNotInPark,
    TorqueConverterClutchLocked,
    VoltageTooHigh,
    VoltageTooLow,
    ReserverdForSpecificConditionsNotCorrect,
    ReservedByExtendedDataLinkSecurityDocumentation,
    IsoSAEReserved(u8),
}

impl From<u8> for UDSError {
    fn from(p: u8) -> Self {
        match p {
            0x10 => Self::GeneralReject,
            0x11 => Self::ServiceNotSupported,
            0x12 => Self::SubFunctionNotSupported,
            0x13 => Self::IncorrectMessageLengthOrInvalidFormat,
            0x14 => Self::ResponseTooLong,
            0x21 => Self::BusyRepeatRequest,
            0x22 => Self::ConditionsNotCorrect,
            0x24 => Self::RequestSequenceError,
            0x25 => Self::NoResponseFromSubnetComponent,
            0x26 => Self::FailurePreventsExecutionOfRequestedAction,
            0x31 => Self::RequestOutOfRange,
            0x33 => Self::SecurityAccessDenied,
            0x35 => Self::InvalidKey,
            0x36 => Self::ExceedNumberOfAttempts,
            0x37 => Self::RequiredTimeDelayNotExpired,
            0x70 => Self::UploadDownloadNotAccepted,
            0x71 => Self::TransferDataSuspended,
            0x72 => Self::GeneralProgrammingFailure,
            0x73 => Self::WrongBlockSequenceCounter,
            0x78 => Self::RequestCorrectlyReceivedResponsePending,
            0x7E => Self::SubFunctionNotSupportedInActiveSession,
            0x7F => Self::ServiceNotSupportedInActiveSession,
            0x81 => Self::RpmTooHigh,
            0x82 => Self::RpmTooLow,
            0x83 => Self::EngineIsRunning,
            0x84 => Self::EngineIsNotRunning,
            0x85 => Self::EngineRunTimeTooLow,
            0x86 => Self::TemperatureTooHigh,
            0x87 => Self::TemperatureTooLow,
            0x88 => Self::VehicleSpeedTooHigh,
            0x89 => Self::VehicleSpeedTooLow,
            0x8A => Self::ThrottleTooHigh,
            0x8B => Self::ThrottleTooLow,
            0x8C => Self::TransmissionRangeNotInNeutral,
            0x8D => Self::TransmissionRangeNotInGear,
            0x8F => Self::BrakeSwitchNotClosed,
            0x90 => Self::ShifterLeverNotInPark,
            0x91 => Self::TorqueConverterClutchLocked,
            0x92 => Self::VoltageTooHigh,
            0x93 => Self::VoltageTooLow,
            (0x94..=0xFE) => Self::ReserverdForSpecificConditionsNotCorrect,
            (0x38..=0x4F) => Self::ReservedByExtendedDataLinkSecurityDocumentation,
            x => Self::IsoSAEReserved(x),
        }
    }
}


pub struct UdsDiagnosticServer<T: ECUCommChannel> {
    phantom: PhantomData<T>,
    ecu_timeout_ms: u32,
    server_running: bool

}


impl<T: ECUCommChannel> BasicECUDiagServer<T> for UdsDiagnosticServer<T> {
    fn start_server_canbus(&mut self, channel: T) -> crate::DiagServerResult<()> {
        todo!()
    }

    fn start_server_kline(&mut self, channel: T) -> crate::DiagServerResult<()> {
        return Err(DiagError::NotSupported) // UDS Does not support KLINE diagnostics, only CAN
    }

    fn update_server_loop(&mut self) {
        todo!()
    }

    fn read_dtcs(&mut self) -> crate::DiagServerResult<alloc::vec::Vec<crate::DTC>> {
        todo!()
    }

    fn clear_dtcs(&mut self) -> crate::DiagServerResult<()> {
        todo!()
    }

    fn stop_server(&mut self) {
        todo!()
    }
}



impl<T: ECUCommChannel> AdvancedECUDiagServer<T> for UdsDiagnosticServer<T> {
    type DiagnosticSessionModes = UDSSessionType;

    type DiagnosticErrors;

    fn enter_session_mode(&mut self, mode: Self::DiagnosticSessionModes) -> crate::DiagServerResult<()> {
        todo!()
    }

    fn execute_custom_pid(&mut self, pid: u8, data: &[u8]) -> crate::DiagServerResult<alloc::vec::Vec<u8>> {
        todo!()
    }
}

