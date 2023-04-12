//! Message transmission wrapper

use crate::{DiagServerResult, dynamic_diag::DynamicDiagSession};

use super::{KWP2000Command};

impl DynamicDiagSession {
    /// Tells the ECU to switch off its normal communication paths with other ECUs.
    /// Normally message transmission will be resumed if the ECU looses power, or if
    /// [Kwp2000DiagnosticServer::enable_normal_message_transmission] is called.
    ///
    /// NOTE: Calling this function can make a car go wild as it can no longer talk to
    /// the ECU being sent this command. Use with CAUTION!
    pub fn kwp_disable_normal_message_transmission(&mut self) -> DiagServerResult<()> {
        self.send_command_with_response(
            KWP2000Command::DisableNormalMessageTransmission,
            &[0x01]
        )
        .map(|_| ())
    }

    /// Tells the ECU to re-enable its normal communication paths with other ECUs.
    pub fn kwp_enable_normal_message_transmission(&mut self) -> DiagServerResult<()> {
        self.send_command_with_response(KWP2000Command::EnableNormalMessageTransmission, &[0x01])
            .map(|_| ())
    }
}
