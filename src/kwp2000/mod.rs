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
    dynamic_diag::{self, DiagSessionMode, DiagAction, EcuNRC, DiagPayload},
};

pub mod error;
mod start_diagnostic_session;
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

pub use start_diagnostic_session::*;
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

use self::error::KWP2000Error;

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



#[derive(Debug)]
pub struct Kwp2000Protocol {}

impl dynamic_diag::DiagProtocol<KWP2000Error> for Kwp2000Protocol {
    

    fn process_req_payload(payload: &[u8]) -> DiagAction {
        match KWP2000Command::from(payload[0]) {
            KWP2000Command::StartDiagnosticSession => DiagAction::SetSessionMode(KwpSessionType::from(payload[1]).into()),
            _ => DiagAction::Other { sid: payload[0], data: payload[1..].to_vec() }
        }
    }

    fn create_tp_msg(response_required: bool) -> DiagPayload {
        DiagPayload::new(
            KWP2000Command::TesterPresent.into(), 
            &[if response_required { 0x01 } else { 0x02 }] 
        )
    }

    fn process_ecu_response(r: &[u8]) -> Result<Vec<u8>, (u8, KWP2000Error)> {
        if r[0] == 0x7F { // [7F, SID, NRC]
            let e = KWP2000Error::from(r[2]);
            Err((r[2], e))
        } else {
            Ok(r.to_vec())
        }
    }

    fn get_basic_session_mode() -> Option<DiagSessionMode> {
        Some(DiagSessionMode {
            id: 0x81,
            tp_require: false,
            name: "Default",
        })
    }

    fn get_protocol_name() -> &'static str {
        "KWP2000(CAN)"
    }
}
