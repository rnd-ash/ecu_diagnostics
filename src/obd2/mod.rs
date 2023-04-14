//! Module for OBD (ISO-9141)

use std::collections::HashMap;
use crate::dynamic_diag::{DiagProtocol, EcuNRC, DiagSessionMode, DiagAction, DiagPayload};

mod data_pids;
mod enumerations;
mod service01;
mod service09;
mod units;

// Exports
pub use data_pids::*;
pub use enumerations::*;
pub use service01::*;
pub use service09::*;
pub use units::*;

/// Function to decode PID support response from ECU
pub(crate) fn decode_pid_response(x: &[u8]) -> Vec<bool> {
    let mut resp: Vec<bool> = Vec::new();
    for b in x {
        let mut mask: u8 = 0b10000000;
        for _ in 0..8 {
            resp.push(b & mask != 0x00);
            mask >>= 1;
        }
    }
    resp
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
/// OBD2 Service IDs
pub enum OBD2Command {
    /// Service 01 - Show current data
    Service01,
    /// Service 02 - Show freeze frame data
    Service02,
    /// Service 03 - Show stored DTCs
    Service03,
    /// Service 04 - Clear stored DTCs
    Service04,
    /// Test results, O2 sensor monitoring (non CAN)
    Service05,
    /// Test results, O2 sensor monitoring (CAN)
    Service06,
    /// Show pending DTCs
    Service07,
    /// Control operation of on-board components
    Service08,
    /// Service 09 - Request vehicle information
    Service09,
    /// Service 0A - Read permanent DTCs
    Service0A,
    /// Custom OBD service. Not 0x10+ is either KWP or UDS!
    Custom(u8),
}

impl From<u8> for OBD2Command {
    fn from(sid: u8) -> Self {
        match sid {
            0x01 => Self::Service01,
            0x02 => Self::Service02,
            0x03 => Self::Service03,
            0x04 => Self::Service04,
            0x05 => Self::Service05,
            0x06 => Self::Service06,
            0x07 => Self::Service07,
            0x08 => Self::Service08,
            0x09 => Self::Service09,
            0x0A => Self::Service0A,
            _ => Self::Custom(sid),
        }
    }
}

impl From<OBD2Command> for u8 {
    fn from(cmd: OBD2Command) -> Self {
        match cmd {
            OBD2Command::Service01 => 0x01,
            OBD2Command::Service02 => 0x02,
            OBD2Command::Service03 => 0x03,
            OBD2Command::Service04 => 0x04,
            OBD2Command::Service05 => 0x05,
            OBD2Command::Service06 => 0x06,
            OBD2Command::Service07 => 0x07,
            OBD2Command::Service08 => 0x08,
            OBD2Command::Service09 => 0x09,
            OBD2Command::Service0A => 0x0A,
            OBD2Command::Custom(x) => x,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
/// Wrapper round OBD2 protocol NRC codes
pub enum OBD2Error {
    /// ECU general reject
    GeneralReject,
    /// Service is not supported in active session.
    /// This is a weird error for OBD as OBD only has
    /// one session mode
    ServiceNotSupportedInActiveSession,
    /// Request message format was incorrect
    FormatIncorrect,
    /// Requested data was out of range
    OutOfRange,
    /// ECU is busy, repeat the request
    BusyRepeatRequest,
    /// ECU is busy, but will respond to the original request shortly
    BusyResponsePending,
    /// Conditions are not correct to execute the request
    ConditionsNotCorrect,
    /// Out of order request in a sequence of request
    RequestSequenceError,
    /// Security access is denied
    SecurityAccessDenied,
    /// Invalid security key
    InvalidKey,
    /// Exceeded the maximum number of attempts at authentication
    ExceedAttempts,
    /// OBD NRC. This can mean different things per OEM
    Custom(u8),
}

impl From<u8> for OBD2Error {
    fn from(p: u8) -> Self {
        match p {
            0x10 => Self::GeneralReject,
            0x11 | 0x12 | 0x7E | 0x7F => Self::ServiceNotSupportedInActiveSession,
            0x13 => Self::FormatIncorrect,
            0x31 => Self::OutOfRange,
            0x21 => Self::BusyRepeatRequest,
            0x78 => Self::BusyResponsePending,
            0x22 => Self::ConditionsNotCorrect,
            0x24 => Self::RequestSequenceError,
            0x33 => Self::SecurityAccessDenied,
            0x35 => Self::InvalidKey,
            0x36 => Self::ExceedAttempts,
            x => Self::Custom(x),
        }
    }
}

impl EcuNRC for OBD2Error {
    fn desc(&self) -> String {
        format!("{:02X?}", self)
    }

    fn is_ecu_busy(&self) -> bool {
        matches!(self, Self::BusyResponsePending)
    }

    fn is_wrong_diag_mode(&self) -> bool {
        matches!(self, Self::ServiceNotSupportedInActiveSession)
    }

    fn is_repeat_request(&self) -> bool {
        matches!(self, Self::BusyRepeatRequest)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// OBD2 diagnostic protocol
pub struct OBD2Protocol{}

impl DiagProtocol<OBD2Error> for OBD2Protocol {
    fn get_basic_session_mode(&self) -> Option<DiagSessionMode> {
        None
    }

    fn get_protocol_name(&self) -> &'static str {
        "OBD2(CAN)"
    }

    fn process_req_payload(&self, payload: &[u8]) -> DiagAction {
        DiagAction::Other { sid: payload[0], data: payload[1..].to_vec() }
    }

    fn create_tp_msg(_response_required: bool) -> DiagPayload {
        DiagPayload::new(0x00, &[]) // Ignored
    }

    fn process_ecu_response(r: &[u8]) -> Result<Vec<u8>, (u8, OBD2Error)> {
        match r[0] {
            0x7F => Err((r[2], OBD2Error::from(r[2]))),
            _ => Ok(r.to_vec())
        }
    }

    fn get_diagnostic_session_list(&self) -> HashMap<u8, DiagSessionMode> {
        HashMap::new()
    }

    fn register_session_type(&mut self, _session: DiagSessionMode) {
    }
}