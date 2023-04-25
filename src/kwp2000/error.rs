use crate::dynamic_diag::EcuNRC;
use automotive_diag::kwp2000::{KwpError, KwpErrorByte};
use automotive_diag::ByteWrapper::Standard;

impl EcuNRC for KwpErrorByte {
    fn desc(&self) -> String {
        format!("{self:?}")
    }

    fn is_ecu_busy(&self) -> bool {
        *self == Standard(KwpError::RequestCorrectlyReceivedResponsePending)
    }

    fn is_wrong_diag_mode(&self) -> bool {
        *self == Standard(KwpError::ServiceNotSupportedInActiveSession)
    }

    fn is_repeat_request(&self) -> bool {
        *self == Standard(KwpError::BusyRepeatRequest)
    }
}
