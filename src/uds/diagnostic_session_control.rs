//!  Provides methods to manipulate the ECUs diagnostic session mode

use crate::{DiagServerResult, dynamic_diag::{DynamicDiagSession, DiagSessionMode}};

use auto_uds::UdsCommand;
pub use auto_uds::UdsSessionType as UDSSessionType;


impl Into<DiagSessionMode> for auto_uds::UdsSessionType {
    fn into(self) -> DiagSessionMode {
        match self {
            UDSSessionType::Default => DiagSessionMode { 
                id: 0x01, 
                tp_require: false, 
                name: "Default" 
            },
            UDSSessionType::Programming => DiagSessionMode { 
                id: 0x02, 
                tp_require: true, 
                name: "Programming" 
            },
            UDSSessionType::Extended => DiagSessionMode { 
                id: 0x03, 
                tp_require: true, 
                name: "Extended" 
            },
            UDSSessionType::SafetySystem => DiagSessionMode { 
                id: 0x04, 
                tp_require: true, 
                name: "SafetySystem" 
            },
            UDSSessionType::Other(x) => DiagSessionMode { 
                id: x, 
                tp_require: true, 
                name: "Custom" 
            },
        }
    }
}

impl DynamicDiagSession {
    /// Requests the ECU to go into a specific diagnostic session mode
    pub fn uds_set_session_mode(&mut self, session_mode: UDSSessionType) -> DiagServerResult<()> {
        self.send_command_with_response(
            UdsCommand::DiagnosticSessionControl,
            &[session_mode.into()],
        )
        .map(|_| ())
    }
}
