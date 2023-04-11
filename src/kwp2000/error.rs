use crate::dynamic_diag::EcuNRC;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
/// KWP Error definitions
pub enum KWP2000Error {
    /// ECU rejected the request for unknown reason
    GeneralReject,
    /// ECU Does not support the requested service
    ServiceNotSupported,
    /// ECU does not support arguments provided, or message format is incorrect
    SubFunctionNotSupportedInvalidFormat,
    /// ECU is too busy to perform the request
    BusyRepeatRequest,
    /// ECU prerequisite conditions are not met
    ConditionsNotCorrectRequestSequenceError,
    /// **Deprecated in v2.2 of KWP2000**. Requested results of a routine that is not completed.
    RoutineNotComplete,
    /// The request message contains data which is out of range
    RequestOutOfRange,
    /// Security access is denied
    SecurityAccessDenied,
    /// Invalid key provided to the ECU
    InvalidKey,
    /// Exceeded the number of incorrect security access attempts
    ExceedNumberOfAttempts,
    /// Time period for requesting a new seed not expired
    RequiredTimeDelayNotExpired,
    /// ECU fault prevents data download
    DownloadNotAccepted,
    /// ECU fault prevents data upload
    UploadNotAccepted,
    /// ECU fault has stopped the transfer of data
    TransferSuspended,
    /// The ECU has accepted the request, but cannot reply right now. If this error occurs,
    /// the [Kwp2000DiagnosticServer] will automatically stop sending tester present messages and
    /// will wait for the ECUs response. If after 2000ms, the ECU did not respond, then this error
    /// will get returned back to the function call.
    RequestCorrectlyReceivedResponsePending,
    /// Requested service is not supported in the current diagnostic session mode
    ServiceNotSupportedInActiveSession,
    /// Reserved for future ISO14230 use
    ReservedISO,
    /// Reserved for future use by DCX (Daimler)
    ReservedDCX,
    /// Data decompression failed
    DataDecompressionFailed,
    /// Data decryption failed
    DataDecryptionFailed,
    /// Sent by a gateway ECU. The requested ECU behind the gateway is not responding
    EcuNotResponding,
    /// Sent by a gateway ECU. The requested ECU address is unknown
    EcuAddressUnknown,
}

impl From<u8> for KWP2000Error {
    fn from(p: u8) -> Self {
        match p {
            0x10 => Self::GeneralReject,
            0x11 => Self::ServiceNotSupported,
            0x12 => Self::SubFunctionNotSupportedInvalidFormat,
            0x21 => Self::BusyRepeatRequest,
            0x22 => Self::ConditionsNotCorrectRequestSequenceError,
            0x23 => Self::RoutineNotComplete,
            0x31 => Self::RequestOutOfRange,
            0x33 => Self::SecurityAccessDenied,
            0x35 => Self::InvalidKey,
            0x36 => Self::ExceedNumberOfAttempts,
            0x37 => Self::RequiredTimeDelayNotExpired,
            0x40 => Self::DownloadNotAccepted,
            0x50 => Self::UploadNotAccepted,
            0x71 => Self::TransferSuspended,
            0x78 => Self::RequestCorrectlyReceivedResponsePending,
            0x80 => Self::ServiceNotSupportedInActiveSession,
            0x90..=0x99 => Self::ReservedDCX,
            0x9A => Self::DataDecompressionFailed,
            0x9B => Self::DataDecryptionFailed,
            0x9C..=0x9F => Self::ReservedDCX,
            0xA0 => Self::EcuNotResponding,
            0xA1 => Self::EcuAddressUnknown,
            0xA2..=0xF9 => Self::ReservedDCX,
            _ => Self::ReservedISO,
        }
    }
}

impl EcuNRC for KWP2000Error {
    fn desc(&self) -> String {
        format!("{:?}", self)
    }

    fn is_ecu_busy(&self) -> bool {
        *self == KWP2000Error::BusyRepeatRequest
    }

    fn is_wrong_diag_mode(&self) -> bool {
        *self == KWP2000Error::ServiceNotSupportedInActiveSession
    }

    fn is_repeat_request(&self) -> bool {
        *self == KWP2000Error::BusyRepeatRequest
    }
}