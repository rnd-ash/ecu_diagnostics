use std::time::{Duration, Instant};

use crate::{BaseChannel, BaseServerPayload, BaseServerSettings, DiagError, DiagServerResult};

/// Checks if the response payload matches the request ServiceID.
/// For both KWP and UDS, the matching response SID is request + 0x40.
///
/// ## Parameters
/// * sid - The SID to match against
/// * resp - Response from the ECU to check
pub fn check_pos_response_id(sid: u8, resp: Vec<u8>) -> DiagServerResult<Vec<u8>> {
    if resp[0] != sid + 0x40 {
        Err(DiagError::WrongMessage)
    } else {
        Ok(resp)
    }
}

pub fn perform_cmd<P: BaseServerPayload, T: BaseServerSettings>(
    cmd: &P,
    settings: &T,
    channel: &mut Box<dyn BaseChannel>,
    await_response_bytes: u8,
) -> DiagServerResult<Vec<u8>> {
    // Clear IO buffers
    channel.clear_rx_buffer()?;
    channel.clear_tx_buffer()?;
    let target = cmd.get_sid_byte();
    if !cmd.requires_response() {
        // Just send the data and return an empty response
        channel.write_bytes(cmd.to_bytes(), settings.get_write_timeout_ms())?;
        return Ok(Vec::new());
    }
    let res = channel.read_write_bytes(
        &cmd.to_bytes(),
        settings.get_write_timeout_ms(),
        settings.get_read_timeout_ms(),
    )?;
    if res.is_empty() {
        return Err(DiagError::EmptyResponse);
    }
    if res[0] == 0x7F {
        if res[1] == await_response_bytes {
            // For both UDS or
            // Wait a bit longer for the ECU response
            let timestamp = Instant::now();
            while timestamp.elapsed() <= Duration::from_millis(1000) {
                std::thread::sleep(std::time::Duration::from_millis(10));
                if let Ok(res2) = channel.read_bytes(settings.get_read_timeout_ms()) {
                    if res2.is_empty() {
                        return Err(DiagError::EmptyResponse);
                    }
                    if res2[0] == 0x7F {
                        // Still an error. Give up
                        return Err(super::DiagError::ECUError(res2[1].into()));
                    } else {
                        // Response OK! Set last tester time so we don't flood the ECU too quickly
                        return check_pos_response_id(target, res2);
                    }
                }
            }
            // No response! Return the last error
            return Err(super::DiagError::ECUError(res[1].into()));
        } else {
            // Other error! - Return that error
            return Err(super::DiagError::ECUError(res[1].into()));
        }
    }
    check_pos_response_id(target, res) // ECU Response OK!
}
