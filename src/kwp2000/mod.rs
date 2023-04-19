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

pub use automotive_diag::kwp2000::*;
use std::collections::HashMap;

use crate::dynamic_diag::{self, DiagAction, DiagPayload, DiagSessionMode};

mod clear_diagnostic_information;
mod ecu_reset;
mod error;
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
pub use error::*;
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

#[derive(Debug, Clone)]
/// KWP2000 diagnostic protocol
pub struct Kwp2000Protocol {
    session_modes: HashMap<u8, DiagSessionMode>,
}

impl Default for Kwp2000Protocol {
    /// Creates a new KWP2000 protocol with standard session types
    fn default() -> Kwp2000Protocol {
        let mut session_modes = HashMap::new();
        session_modes.insert(
            0x81,
            DiagSessionMode {
                id: 0x81,
                tp_require: false,
                name: "Normal".into(),
            },
        );
        session_modes.insert(
            0x85,
            DiagSessionMode {
                id: 0x85,
                tp_require: true,
                name: "Reprogramming".into(),
            },
        );
        session_modes.insert(
            0x89,
            DiagSessionMode {
                id: 0x89,
                tp_require: true,
                name: "Standby".into(),
            },
        );
        session_modes.insert(
            0x90,
            DiagSessionMode {
                id: 0x90,
                tp_require: false,
                name: "Passive".into(),
            },
        );
        session_modes.insert(
            0x92,
            DiagSessionMode {
                id: 0x92,
                tp_require: true,
                name: "ExtendedDiagnostics".into(),
            },
        );
        Kwp2000Protocol { session_modes }
    }
}

impl dynamic_diag::DiagProtocol<KwpErrorByte> for Kwp2000Protocol {
    fn process_req_payload(&self, payload: &[u8]) -> DiagAction {
        if matches!(
            KwpCommand::try_from(payload[0]),
            Ok(KwpCommand::StartDiagnosticSession)
        ) {
            DiagAction::SetSessionMode(
                self.session_modes
                    .get(&payload[1])
                    .unwrap_or(&DiagSessionMode {
                        id: payload[1],
                        tp_require: true,
                        name: format!("Unkown(0x{:02X?})", payload[1]),
                    })
                    .clone(),
            )
        } else if matches!(
            KwpCommand::try_from(payload[0]),
            Ok(KwpCommand::ECUReset)
        ) {
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
            KwpCommand::TesterPresent.into(),
            &[if response_required { 0x01 } else { 0x02 }],
        )
    }

    fn make_session_control_msg(&self, mode: &DiagSessionMode) -> Vec<u8> {
        vec![KwpCommand::StartDiagnosticSession.into(), mode.id]
    }

    fn process_ecu_response(r: &[u8]) -> Result<Vec<u8>, (u8, KwpErrorByte)> {
        if r[0] == 0x7F {
            // [7F, SID, NRC]
            Err((r[2], r[2].into()))
        } else {
            Ok(r.to_vec())
        }
    }

    fn get_basic_session_mode(&self) -> Option<DiagSessionMode> {
        self.session_modes.get(&0x81).cloned()
    }

    fn get_protocol_name(&self) -> &'static str {
        "KWP2000(CAN)"
    }

    fn get_diagnostic_session_list(&self) -> HashMap<u8, DiagSessionMode> {
        self.session_modes.clone()
    }

    fn register_session_type(&mut self, session: DiagSessionMode) {
        self.session_modes.insert(session.id, session);
    }
}
