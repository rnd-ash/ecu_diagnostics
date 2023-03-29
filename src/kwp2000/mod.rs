//! Module for KWP2000 (Keyword protocol 2000 - ISO142330)
//!
//! This module is written to be 100% compliant with the following vehicle manufactures
//! which utilize KWP2000:
//! * Dodge
//! * Chrysler
//! * Jeep
//! * Mitsubishi (Abbreviated as MMC)
//! * Daimler (Mercedes-Benz, Maybach and SMART)
//!
//! Other manufacturer's ECUs might also work, however they are untested.
//!
//! based on KWP2000 v2.2 (05/08/02)

use crate::{
    dynamic_diag::{self, DiagSessionMode, DiagSID, DiagServerRx},
};

mod clear_diagnostic_information;
mod ecu_reset;
mod ioctl_mgr;
mod message_transmission;
mod read_data_by_identifier;
mod read_data_by_local_id;
mod read_dtc_by_status;
mod read_ecu_identification;
mod read_memory_by_address;
mod read_status_of_dtc;
mod routine;
mod security_access;
mod start_diagnostic_session;

pub use clear_diagnostic_information::*;
pub use ecu_reset::*;
pub use ioctl_mgr::*;
pub use message_transmission::*;
pub use read_data_by_identifier::*;
pub use read_data_by_local_id::*;
pub use read_dtc_by_status::*;
pub use read_ecu_identification::*;
pub use read_memory_by_address::*;
pub use read_status_of_dtc::*;
pub use routine::*;
pub use security_access::*;
pub use start_diagnostic_session::*;

/// KWP Command Service IDs.
///
/// Note. This does not cover both the 'Reserved' range (0x87-0xB9) and
/// 'System supplier specific' range (0xBA-0xBF)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum KWP2000Command {
    /// Start or change ECU diagnostic session mode.
    StartDiagnosticSession,
    /// Reset the ECU.
    ECUReset,
    /// Clears diagnostic information stored on the ECU.
    ClearDiagnosticInformation,
    /// Reads snapshot data of DTCs stored on the ECU.
    ReadStatusOfDiagnosticTroubleCodes,
    /// Reads DTCs stored on the ECU.
    ReadDiagnosticTroubleCodesByStatus,
    /// Reads ECU identification data.
    ReadECUIdentification,
    /// Reads data from the ECU using a local identifier.
    ReadDataByLocalIdentifier,
    /// Reads data from the ECU using a unique identifier.
    ReadDataByIdentifier,
    /// Reads memory from the ECU by address.
    ReadMemoryByAddress,
    /// Security access functions.
    SecurityAccess,
    /// Disables normal CAN message transmission from an ECU.
    DisableNormalMessageTransmission,
    /// Enables normal CAN message transmission from an ECU.
    EnableNormalMessageTransmission,
    ///
    DynamicallyDefineLocalIdentifier,
    ///
    WriteDataByIdentifier,
    ///
    InputOutputControlByLocalIdentifier,
    /// Starts a ECU routine given a local identifier.
    StartRoutineByLocalIdentifier,
    /// Stops a ECU routine given a local identifier.
    StopRoutineByLocalIdentifier,
    /// requests results of an executed routine given a local identifier.
    RequestRoutineResultsByLocalIdentifier,
    ///
    RequestDownload,
    ///
    RequestUpload,
    ///
    TransferData,
    ///
    RequestTransferExit,
    ///
    WriteDataByLocalIdentifier,
    ///
    WriteMemoryByAddress,
    /// Tester present message. [Kwp2000DiagnosticServer] will automatically send this,
    /// so no need to manually create a message with this SID
    TesterPresent,
    ///
    ControlDTCSettings,
    ///
    ResponseOnEvent,
    /// Custom KWP2000 SID not part of the official specification
    CustomSid(u8),
}

impl DiagSID for KWP2000Command {

}

impl From<u8> for KWP2000Command {
    fn from(sid: u8) -> Self {
        match sid {
            0x10 => KWP2000Command::StartDiagnosticSession,
            0x11 => KWP2000Command::ECUReset,
            0x14 => KWP2000Command::ClearDiagnosticInformation,
            0x17 => KWP2000Command::ReadStatusOfDiagnosticTroubleCodes,
            0x18 => KWP2000Command::ReadDiagnosticTroubleCodesByStatus,
            0x1A => KWP2000Command::ReadECUIdentification,
            0x21 => KWP2000Command::ReadDataByLocalIdentifier,
            0x22 => KWP2000Command::ReadDataByIdentifier,
            0x23 => KWP2000Command::ReadMemoryByAddress,
            0x27 => KWP2000Command::SecurityAccess,
            0x28 => KWP2000Command::DisableNormalMessageTransmission,
            0x29 => KWP2000Command::EnableNormalMessageTransmission,
            0x2C => KWP2000Command::DynamicallyDefineLocalIdentifier,
            0x2E => KWP2000Command::WriteDataByIdentifier,
            0x30 => KWP2000Command::InputOutputControlByLocalIdentifier,
            0x31 => KWP2000Command::StartRoutineByLocalIdentifier,
            0x32 => KWP2000Command::StopRoutineByLocalIdentifier,
            0x33 => KWP2000Command::RequestRoutineResultsByLocalIdentifier,
            0x34 => KWP2000Command::RequestDownload,
            0x35 => KWP2000Command::RequestUpload,
            0x36 => KWP2000Command::TransferData,
            0x37 => KWP2000Command::RequestTransferExit,
            0x3B => KWP2000Command::WriteDataByLocalIdentifier,
            0x3D => KWP2000Command::WriteMemoryByAddress,
            0x3E => KWP2000Command::TesterPresent,
            0x85 => KWP2000Command::ControlDTCSettings,
            0x86 => KWP2000Command::ResponseOnEvent,
            s => KWP2000Command::CustomSid(s),
        }
    }
}

impl From<KWP2000Command> for u8 {
    fn from(cmd: KWP2000Command) -> Self {
        match cmd {
            KWP2000Command::StartDiagnosticSession => 0x10,
            KWP2000Command::ECUReset => 0x11,
            KWP2000Command::ClearDiagnosticInformation => 0x14,
            KWP2000Command::ReadStatusOfDiagnosticTroubleCodes => 0x17,
            KWP2000Command::ReadDiagnosticTroubleCodesByStatus => 0x18,
            KWP2000Command::ReadECUIdentification => 0x1A,
            KWP2000Command::ReadDataByLocalIdentifier => 0x21,
            KWP2000Command::ReadDataByIdentifier => 0x22,
            KWP2000Command::ReadMemoryByAddress => 0x23,
            KWP2000Command::SecurityAccess => 0x27,
            KWP2000Command::DisableNormalMessageTransmission => 0x28,
            KWP2000Command::EnableNormalMessageTransmission => 0x29,
            KWP2000Command::DynamicallyDefineLocalIdentifier => 0x2C,
            KWP2000Command::WriteDataByIdentifier => 0x2E,
            KWP2000Command::InputOutputControlByLocalIdentifier => 0x30,
            KWP2000Command::StartRoutineByLocalIdentifier => 0x31,
            KWP2000Command::StopRoutineByLocalIdentifier => 0x32,
            KWP2000Command::RequestRoutineResultsByLocalIdentifier => 0x33,
            KWP2000Command::RequestDownload => 0x34,
            KWP2000Command::RequestUpload => 0x35,
            KWP2000Command::TransferData => 0x36,
            KWP2000Command::RequestTransferExit => 0x37,
            KWP2000Command::WriteDataByLocalIdentifier => 0x3B,
            KWP2000Command::WriteMemoryByAddress => 0x3D,
            KWP2000Command::TesterPresent => 0x3E,
            KWP2000Command::ControlDTCSettings => 0x85,
            KWP2000Command::ResponseOnEvent => 0x86,
            KWP2000Command::CustomSid(s) => s,
        }
    }
}

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

fn lookup_kwp_nrc(x: u8) -> String {
    format!("{:?}", KWP2000Error::from(x))
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

#[derive(Debug)]
pub struct Kwp2000Protocol {
}

impl Kwp2000Protocol {
}

impl dynamic_diag::DiagProtocol for Kwp2000Protocol {
    fn get_basic_session_mode(&self) -> DiagSessionMode {
        DiagSessionMode {
            id: 0x81,
            tp_require: false,
            name: "Default",
        }
    }

    fn get_protocol_name(&self) -> &'static str {
        "KWP2000"
    }

    fn process_req_payload(payload: &[u8]) -> dynamic_diag::DiagAction {
        todo!()
    }

    fn create_tp_msg(response_required: bool) -> dynamic_diag::DiagAction {
        todo!()
    }

    fn process_ecu_response(r: &[u8]) -> DiagServerRx {
        if r[0] == 0x7F {
            if r[1] == 0x78 {
                DiagServerRx::EcuWaiting
            } else {
                DiagServerRx::EcuError(r[1])
            }
        } else {
            DiagServerRx::EcuResponse(r.to_vec())
        }
    }
}
