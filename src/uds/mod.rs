//! Module for UDS (Unified diagnostic services - ISO14229)
//!
//! Theoretically, this module should be compliant with any ECU which implements
//! UDS (Typically any ECU produced after 2006 supports this)

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, RwLock,
    },
    time::Instant,
};

use crate::{dynamic_diag::{DiagProtocol, EcuNRC, DiagSessionMode, DiagAction, DiagPayload}};

mod access_timing_parameter;
mod clear_diagnostic_information;
mod communication_control;
mod diagnostic_session_control;
mod ecu_reset;
mod read_dtc_information;
mod scaling_data;
mod security_access;

pub use access_timing_parameter::*;
use auto_uds::UdsCommand;
pub use clear_diagnostic_information::*;
pub use communication_control::*;
pub use diagnostic_session_control::*;
pub use ecu_reset::*;
pub use read_dtc_information::*;
pub use scaling_data::*;
pub use security_access::*;

pub use auto_uds::{UdsError};

pub struct UDSErrorWrapper(UdsError);

impl From<u8> for UDSErrorWrapper {
    fn from(value: u8) -> Self {
        Self(UdsError::from(value))
    }
}

impl EcuNRC for UDSErrorWrapper {
    fn desc(&self) -> String {
        format!("{:?}", self.0)
    }

    fn is_ecu_busy(&self) -> bool {
        self.0 == UdsError::RequestCorrectlyReceivedResponsePending
    }

    fn is_wrong_diag_mode(&self) -> bool {
        self.0 == UdsError::ServiceNotSupportedInActiveSession
    }

    fn is_repeat_request(&self) -> bool {
        self.0 == UdsError::BusyRepeatRequest
    }
}

pub struct UDSProtocol{}

impl DiagProtocol<UDSErrorWrapper> for UDSProtocol {
    fn get_basic_session_mode() -> Option<crate::dynamic_diag::DiagSessionMode> {
        Some(
            DiagSessionMode {
                id: auto_uds::UdsSessionType::Default.into(),
                tp_require: false,
                name: "Default",
            }
        )
    }

    fn get_protocol_name() -> &'static str {
        "UDS"
    }

    fn process_req_payload(payload: &[u8]) -> crate::dynamic_diag::DiagAction {
        match payload[0] {
            0x10 => {
                let mode = match payload[1] {
                    0x01 => UDSSessionType::Default,
                    0x02 => UDSSessionType::Programming,
                    0x03 => UDSSessionType::Extended,
                    0x04 => UDSSessionType::SafetySystem,
                    x => UDSSessionType::Other(x)
                };
                DiagAction::SetSessionMode(mode.into())
            },
            x => DiagAction::Other { sid: x, data: payload[1..].to_vec() }
        }
    }

    fn create_tp_msg(response_required: bool) -> crate::dynamic_diag::DiagPayload {
        DiagPayload::new(UdsCommand::TesterPresent.into(), &[if response_required {0x00} else {0x80}])
    }

    fn process_ecu_response(r: &[u8]) -> Result<Vec<u8>, (u8, UDSErrorWrapper)> {
        if r[0] == 0x7F { // [7F, SID, NRC]
            Err((r[2], UDSErrorWrapper::from(r[2])))
        } else {
            Ok(r.to_vec())
        }
    }
}
