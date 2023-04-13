//! Module for UDS (Unified diagnostic services - ISO14229)
//!
//! Theoretically, this module should be compliant with any ECU which implements
//! UDS (Typically any ECU produced after 2006 supports this)

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, RwLock,
    },
    time::Instant, collections::HashMap, fmt::format,
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

pub struct UDSProtocol{
    session_modes: HashMap<u8, DiagSessionMode>
}

impl UDSProtocol {
    pub fn new() -> Self {
        let mut session_modes = HashMap::new();
        session_modes.insert(0x01, DiagSessionMode { id: 0x01, tp_require: false, name: "Default".into() });
        session_modes.insert(0x02, DiagSessionMode { id: 0x02, tp_require: true, name: "Programming".into() });
        session_modes.insert(0x03, DiagSessionMode { id: 0x03, tp_require: true, name: "Extended".into() });
        session_modes.insert(0x04, DiagSessionMode { id: 0x04, tp_require: true, name: "SafetySystem".into() });
        Self {
            session_modes
        }
    }
}

impl DiagProtocol<UDSErrorWrapper> for UDSProtocol {
    fn get_basic_session_mode(&self) -> Option<crate::dynamic_diag::DiagSessionMode> {
        self.session_modes.get(&UDSSessionType::Default.into()).cloned()
    }

    fn get_protocol_name(&self) -> &'static str {
        "UDS"
    }

    fn process_req_payload(&self, payload: &[u8]) -> crate::dynamic_diag::DiagAction {
        match payload[0] {
            0x10 => {
                let mode = self.session_modes.get(&payload[1]).unwrap_or(&DiagSessionMode {
                    id: payload[1],
                    tp_require: true,
                    name: format!("Unknown (0x{:02X?})", payload[1])
                });
                DiagAction::SetSessionMode(*mode)
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

    fn get_diagnostic_session_list(&self) -> std::collections::HashMap<u8, DiagSessionMode> {
        todo!()
    }

    fn register_session_type(&mut self, session: DiagSessionMode) {
        todo!()
    }
}
