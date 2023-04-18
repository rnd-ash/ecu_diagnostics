//! Module for OBD (ISO-9141)

use crate::dynamic_diag::{DiagAction, DiagPayload, DiagProtocol, DiagSessionMode, EcuNRC};
use automotive_diag::obd2::{Obd2Error, Obd2ErrorByte};
use automotive_diag::ByteWrapper::Standard;
use std::collections::HashMap;

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

impl EcuNRC for Obd2ErrorByte {
    fn desc(&self) -> String {
        format!("{:02X?}", self)
    }

    fn is_ecu_busy(&self) -> bool {
        matches!(self, Standard(Obd2Error::BusyResponsePending))
    }

    fn is_wrong_diag_mode(&self) -> bool {
        self.is_not_supported()
    }

    fn is_repeat_request(&self) -> bool {
        matches!(self, Standard(Obd2Error::BusyRepeatRequest))
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// OBD2 diagnostic protocol
pub struct OBD2Protocol {}

impl DiagProtocol<Obd2ErrorByte> for OBD2Protocol {
    fn get_basic_session_mode(&self) -> Option<DiagSessionMode> {
        None
    }

    fn get_protocol_name(&self) -> &'static str {
        "OBD2(CAN)"
    }

    fn process_req_payload(&self, payload: &[u8]) -> DiagAction {
        DiagAction::Other {
            sid: payload[0],
            data: payload[1..].to_vec(),
        }
    }

    fn create_tp_msg(_response_required: bool) -> DiagPayload {
        DiagPayload::new(0x00, &[]) // Ignored
    }

    fn make_session_control_msg(&self, mode: &DiagSessionMode) -> Vec<u8> {
        vec![]
    }

    fn process_ecu_response(r: &[u8]) -> Result<Vec<u8>, (u8, Obd2ErrorByte)> {
        match r[0] {
            0x7F => Err((r[2], r[2].into())),
            _ => Ok(r.to_vec()),
        }
    }

    fn get_diagnostic_session_list(&self) -> HashMap<u8, DiagSessionMode> {
        HashMap::new()
    }

    fn register_session_type(&mut self, _session: DiagSessionMode) {}
}
