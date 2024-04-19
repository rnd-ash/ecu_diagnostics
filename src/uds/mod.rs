//! Module for UDS (Unified diagnostic services - ISO14229)
//!
//! Theoretically, this module should be compliant with any ECU which implements
//! UDS (Typically any ECU produced after 2006 supports this)

use crate::dynamic_diag::{DiagAction, DiagPayload, DiagProtocol, DiagSessionMode, EcuNRC};
use std::collections::HashMap;

mod access_timing_parameter;
mod clear_diagnostic_information;
mod communication_control;
mod diagnostic_session_control;
mod ecu_reset;
mod read_dtc_information;
mod scaling_data;
mod security_access;

pub use automotive_diag::uds::*;
use automotive_diag::ByteWrapper;
pub use communication_control::*;
pub use scaling_data::*;

pub use automotive_diag::uds::UdsError;

impl EcuNRC for UdsErrorByte {
    fn desc(&self) -> String {
        match self {
            ByteWrapper::Standard(e) => format!("{e:?}"),
            ByteWrapper::Extended(b) => format!("Unknown error code 0x{b:02X?}"),
        }
    }

    fn is_ecu_busy(&self) -> bool {
        matches!(
            self,
            UdsErrorByte::Standard(UdsError::RequestCorrectlyReceivedResponsePending)
        )
    }

    fn is_wrong_diag_mode(&self) -> bool {
        matches!(
            self,
            UdsErrorByte::Standard(UdsError::ServiceNotSupportedInActiveSession)
        )
    }

    fn is_repeat_request(&self) -> bool {
        matches!(self, UdsErrorByte::Standard(UdsError::BusyRepeatRequest))
    }
}

#[derive(Debug, Clone)]
/// UDS diagnostic protocol
pub struct UDSProtocol {
    session_modes: HashMap<u8, DiagSessionMode>,
}

impl Default for UDSProtocol {
    /// Creates a new UDS protocol, and enables standard session types
    fn default() -> Self {
        let mut session_modes = HashMap::new();
        session_modes.insert(
            0x01,
            DiagSessionMode {
                id: 0x01,
                tp_require: false,
                name: "Default".into(),
            },
        );
        session_modes.insert(
            0x02,
            DiagSessionMode {
                id: 0x02,
                tp_require: true,
                name: "Programming".into(),
            },
        );
        session_modes.insert(
            0x03,
            DiagSessionMode {
                id: 0x03,
                tp_require: true,
                name: "Extended".into(),
            },
        );
        session_modes.insert(
            0x04,
            DiagSessionMode {
                id: 0x04,
                tp_require: true,
                name: "SafetySystem".into(),
            },
        );
        Self { session_modes }
    }
}

impl DiagProtocol<ByteWrapper<UdsError>> for UDSProtocol {
    fn get_basic_session_mode(&self) -> Option<DiagSessionMode> {
        self.session_modes
            .get(&UdsSessionType::Default.into())
            .cloned()
    }

    fn get_protocol_name(&self) -> &'static str {
        "UDS"
    }

    fn process_req_payload(&self, payload: &[u8]) -> DiagAction {
        let pid = UdsCommandByte::from(payload[0]);
        if matches!(
            pid,
            UdsCommandByte::Standard(UdsCommand::DiagnosticSessionControl)
        ) {
            let mode = self
                .session_modes
                .get(&payload[1])
                .unwrap_or(&DiagSessionMode {
                    id: payload[1],
                    tp_require: true,
                    name: format!("Unknown (0x{:02X?})", payload[1]),
                })
                .clone();
            DiagAction::SetSessionMode(mode)
        } else if matches!(pid, UdsCommandByte::Standard(UdsCommand::ECUReset)) {
            DiagAction::EcuReset
        } else {
            DiagAction::Other {
                sid: payload[0],
                data: payload[1..].to_vec(),
            }
        }
    }

    fn create_tp_msg(response_required: bool) -> DiagPayload {
        DiagPayload::new(
            UdsCommand::TesterPresent.into(),
            &[if response_required { 0x00 } else { 0x80 }],
        )
    }

    fn make_session_control_msg(&self, mode: &DiagSessionMode) -> Vec<u8> {
        vec![UdsCommand::DiagnosticSessionControl.into(), mode.id]
    }

    fn process_ecu_response(r: &[u8]) -> Result<Vec<u8>, (u8, UdsErrorByte)> {
        if r[0] == 0x7F {
            // [7F, SID, NRC]
            // OR (Bootloader sometimes) [0x7F, NRC]
            Err((*r.last().unwrap(), UdsErrorByte::from(*r.last().unwrap())))
        } else {
            Ok(r.to_vec())
        }
    }

    fn get_diagnostic_session_list(&self) -> HashMap<u8, DiagSessionMode> {
        self.session_modes.clone()
    }

    fn register_session_type(&mut self, session: DiagSessionMode) {
        self.session_modes.insert(session.id, session);
    }
}
