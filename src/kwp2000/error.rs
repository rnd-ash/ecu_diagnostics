use crate::dynamic_diag::EcuNRC;
use auto_uds::kwp2k::{KwpError, KwpErrorByte};
use auto_uds::ByteWrapper::Standard;

impl EcuNRC for KwpErrorByte {
    fn desc(&self) -> String {
        format!("{self:?}")
    }

    fn is_ecu_busy(&self) -> bool {
        *self == Standard(KwpError::BusyRepeatRequest)
    }

    fn is_wrong_diag_mode(&self) -> bool {
        *self == Standard(KwpError::ServiceNotSupportedInActiveSession)
    }

    fn is_repeat_request(&self) -> bool {
        *self == Standard(KwpError::BusyRepeatRequest)
    }
}
