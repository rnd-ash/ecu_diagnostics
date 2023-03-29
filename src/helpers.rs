//! When parsing ECU response data from raw bytes,
//! Some functions in here can be useful with data transformation
use std::time::{Duration, Instant};

use crate::{
    channel::PayloadChannel, BaseServerPayload, BaseServerSettings, DiagError, DiagServerResult, dynamic_diag::{DiagTxPayload, DiagServerBasicOptions, DiagProtocol},
};

/// Checks if the response payload matches the request ServiceID.
/// For both KWP and UDS, the matching response SID is request + 0x40.
///
/// ## Parameters
/// * sid - The SID to match against
/// * resp - Response from the ECU to check
pub(crate) fn check_pos_response_id(sid: u8, resp: Vec<u8>) -> DiagServerResult<Vec<u8>> {
    if resp[0] != sid + 0x40 {
        log::error!(
            "ECU SID mismatch. Request SID was 0x{:02X}, response SID was {:02X?}",
            sid,
            resp[0]
        );
        Err(DiagError::WrongMessage)
    } else {
        log::debug!("ECU SID matches request");
        Ok(resp)
    }
}

pub(crate) fn perform_cmd<
    T: BaseServerSettings,
    C: PayloadChannel,
    P: DiagProtocol
>(
    addr: u32,
    cmd: DiagTxPayload,
    settings: &DiagServerBasicOptions,
    channel: &mut C
) -> DiagServerResult<Vec<u8>> {
    // Clear IO buffers
    channel.clear_tx_buffer()?;
    channel.clear_rx_buffer()?;
    let target = cmd.get_sid_byte();
    if !cmd.requires_response() {
        // Just send the data and return an empty response
        log::debug!("Request doesn't require response. Just sending");
        channel.write_bytes(addr, cmd.to_bytes(), settings.get_write_timeout_ms())?;
        return Ok(Vec::new());
    }
    let res = channel.read_write_bytes(
        addr,
        cmd.to_bytes(),
        settings.get_write_timeout_ms(),
        settings.get_read_timeout_ms(),
    )?;
    log::debug!("ECU response: {:02X?}", res);
    if res.is_empty() {
        return Err(DiagError::EmptyResponse);
    }
    if res[0] == 0x7F {
        if res[2] == P:: {
            log::warn!("ECU Responded with busy_repeat_request! Retrying in 500ms");
            std::thread::sleep(std::time::Duration::from_millis(500));
            return perform_cmd(addr, cmd, settings, channel, busy_repeat_byte, lookup_func);
        }
        if res[2] == 0x78 {
            // Always busy wait for response
            log::warn!("ECU Responded with await_response! Waiting for real response");
            // For both UDS or
            // Wait a bit longer for the ECU response
            let mut timestamp = Instant::now();
            while timestamp.elapsed() <=  Duration::from_millis(settings.get_read_timeout_ms() as u64 * 2) {
                if let Ok(res2) = channel.read_bytes(settings.get_read_timeout_ms()) {
                    log::debug!("ECU next response: {:02X?}", res2);
                    if res2.is_empty() {
                        log::error!("ECU Response was empty after await_response!?");
                        return Err(DiagError::EmptyResponse);
                    }
                    if res2[0] == 0x7F {
                        if res2[2] == 0x78 {
                            log::warn!("ECU Still asking for tester to wait for its response");
                            timestamp = Instant::now();
                        } else {
                            // Still an error but not busy. Give up
                            log::error!("ECU Still responded negatively (0x{:02X?}) after await_response. Giving up.", res2[2]);
                            return Err(super::DiagError::ECUError {
                                code: res2[2],
                                def: Some(lookup_func(res2[2])),
                            })
                        }
                    } else {
                        // Response OK!
                        log::debug!("ECU Responded positively after await.");
                        return check_pos_response_id(target, res2)
                    };
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        log::error!("ECU Negative response 0x{:02X?}", res[2]);
        return Err(super::DiagError::ECUError {
            code: res[2],
            def: Some(lookup_func(res[2])),
        });
    }
    log::debug!("ECU Responded with {:02X?}", res);
    check_pos_response_id(target, res) // ECU Response OK!
}
