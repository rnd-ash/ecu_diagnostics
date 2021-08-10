use std::time::{Duration, Instant};

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
        Err(DiagError::WrongMessage)
    } else {
        Ok(resp)
    }
}

pub(crate) fn perform_cmd<P: BaseServerPayload, T: BaseServerSettings, C: PayloadChannel>(
    addr: u32,
    cmd: &P,
    settings: &T,
    channel: &mut C,
    await_response_byte: u8,
    busy_repeat_byte: u8
) -> DiagServerResult<Vec<u8>> {
    // Clear IO buffers
    channel.clear_rx_buffer()?;
    channel.clear_tx_buffer()?;
    let target = cmd.get_sid_byte();
    if !cmd.requires_response() {
        // Just send the data and return an empty response
        channel.write_bytes(addr, cmd.to_bytes(), settings.get_write_timeout_ms())?;
        return Ok(Vec::new());
    }
    let res = channel.read_write_bytes(
        addr,
        &cmd.to_bytes(),
        settings.get_write_timeout_ms(),
        settings.get_read_timeout_ms(),
    )?;
    if res.is_empty() {
        return Err(DiagError::EmptyResponse);
    }
    if res[0] == 0x7F {
        if res[2] == busy_repeat_byte { // Wait 100ms and retry
            println!("Repeating request!");
            std::thread::sleep(std::time::Duration::from_millis(500));
            return perform_cmd(addr, cmd, settings, channel, await_response_byte, busy_repeat_byte)
        }
        if res[2] == await_response_byte {
            // For both UDS or
            // Wait a bit longer for the ECU response
            let timestamp = Instant::now();
            while timestamp.elapsed() <= Duration::from_millis(4000) {
                if let Ok(res2) = channel.read_bytes(settings.get_read_timeout_ms()) {
                    if res2.is_empty() {
                        return Err(DiagError::EmptyResponse);
                    }
                    if res2[0] == 0x7F {
                        // Still an error. Give up
                        return Err(super::DiagError::ECUError(res2[2]));
                    } else {
                        // Response OK! Set last tester time so we don't flood the ECU too quickly
                        return check_pos_response_id(target, res2);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        return Err(super::DiagError::ECUError(res[2]));
    }
    check_pos_response_id(target, res) // ECU Response OK!
}


pub (crate) fn bcd_decode(input: u8) -> String {
    format!("{}{}", (input & 0xF0) >> 4, input & 0x0F)
}

pub (crate) fn bcd_decode_slice(input: &[u8], sep: Option<&str>) -> String {
    let mut res = String::new();
    for (pos, x) in input.iter().enumerate() {
        res.push_str(bcd_decode(*x).as_str());
        if let Some(separator) = sep {
            if pos != input.len()-1 {
                res.push_str(separator)
            }
        }
    }
    res
}