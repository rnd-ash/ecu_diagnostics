//! When parsing ECU response data from raw bytes,
//! Some functions in here can be useful with data transformation

use std::time::{Duration, Instant};
use log::{debug, error, log_enabled, info, Level, warn};

use crate::{
    channel::PayloadChannel, BaseServerPayload, BaseServerSettings, DiagError, DiagServerResult,
};

/// Checks if the response payload matches the request ServiceID.
/// For both KWP and UDS, the matching response SID is request + 0x40.
///
/// ## Parameters
/// * sid - The SID to match against
/// * resp - Response from the ECU to check
pub(crate) fn check_pos_response_id(sid: u8, resp: Vec<u8>) -> DiagServerResult<Vec<u8>> {
    if resp[0] != sid + 0x40 {
        log::error!("ECU SID mismatch. Request SID was 0x{:02X}, response SID was {:02X?}", sid, resp[0]);
        Err(DiagError::WrongMessage)
    } else {
        log::debug!("ECU SID matches request");
        Ok(resp)
    }
}

pub(crate) fn perform_cmd<P: BaseServerPayload, T: BaseServerSettings, C: PayloadChannel, L: FnOnce(u8) -> String>(
    addr: u32,
    cmd: &P,
    settings: &T,
    channel: &mut C,
    await_response_byte: u8,
    busy_repeat_byte: u8,
    lookup_func: L
) -> DiagServerResult<Vec<u8>> {

    // Clear IO buffers
    channel.clear_rx_buffer()?;
    channel.clear_tx_buffer()?;
    let target = cmd.get_sid_byte();
    if !cmd.requires_response() {
        // Just send the data and return an empty response
        debug!("Request doesn't require response. Just sending.");
        channel.write_bytes(addr, cmd.to_bytes(), settings.get_write_timeout_ms())?;
        return Ok(Vec::new());
    }
    let res = channel.read_write_bytes(
        addr,
        cmd.to_bytes(),
        settings.get_write_timeout_ms(),
        settings.get_read_timeout_ms(),
    )?;
    if res.is_empty() {
        return Err(DiagError::EmptyResponse);
    }
    if res[0] == 0x7F {
        if res[2] == busy_repeat_byte {
            warn!("ECU Responded with busy_repeat_request! Retrying in 500ms");
            std::thread::sleep(std::time::Duration::from_millis(500));
            return perform_cmd(
                addr,
                cmd,
                settings,
                channel,
                await_response_byte,
                busy_repeat_byte,
                lookup_func
            );
        }
        if res[2] == await_response_byte {
            warn!("ECU Responded with await_response! Waiting for real response");
            // For both UDS or
            // Wait a bit longer for the ECU response
            let timestamp = Instant::now();
            while timestamp.elapsed() <= Duration::from_millis(4000) {
                if let Ok(res2) = channel.read_bytes(settings.get_read_timeout_ms()) {
                    if res2.is_empty() {
                        error!("ECU Response was empty after await_response!?");
                        return Err(DiagError::EmptyResponse);
                    }
                    if res2[0] == 0x7F {
                        // Still an error. Give up
                        error!("ECU Still responded negatively (0x{:02X?}) after await_response. Giving up.", res2[2]);
                        return Err(super::DiagError::ECUError { code: res2[2], def: Some(lookup_func(res2[2])) });
                    } else {
                        // Response OK! Set last tester time so we don't flood the ECU too quickly
                        debug!("ECU Responded positivly after await.");
                        return check_pos_response_id(target, res2);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        error!("ECU Negative response 0x{:02X?}", res[2]);
        return Err(super::DiagError::ECUError{ code: res[2], def: Some(lookup_func(res[2])) });
    }
    check_pos_response_id(target, res) // ECU Response OK!
}

/// Converts a single byte into a BCD string
pub fn bcd_decode(input: u8) -> String {
    format!("{}{}", (input & 0xF0) >> 4, input & 0x0F)
}

/// Converts a slice to a BCD string
pub fn bcd_decode_slice(input: &[u8], sep: Option<&str>) -> String {
    let mut res = String::new();
    for (pos, x) in input.iter().enumerate() {
        res.push_str(bcd_decode(*x).as_str());
        if let Some(separator) = sep {
            if pos != input.len() - 1 {
                res.push_str(separator)
            }
        }
    }
    res
}
