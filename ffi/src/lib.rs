//! FFI bindings for ECU_Diagnostics
//!
//! IMPORTANT. Access to the FFI bindings should be done on one thread! No multi-thread support
#![no_std]

extern crate alloc;
extern crate ecu_diagnostics;

use alloc::vec::Vec;

use ecu_diagnostics::{DiagnosticServer, hardware::HardwareError};
pub use ecu_diagnostics::{
    channel::{ChannelError, ChannelResult, IsoTPChannel, IsoTPSettings, PayloadChannel},
    uds::{UDSCommand, UdsDiagnosticServer, UdsServerOptions, UdsVoidHandler},
    DiagError,
};

#[repr(C)]
#[derive(Debug)]
/// Callback handler payload
pub struct CallbackPayload {
    /// Target send address
    pub addr: u32,
    /// Data size
    pub data_len: u32,
    /// Data pointer
    pub data: *const u8,
}

impl Default for CallbackPayload {
    fn default() -> Self {
        Self {
            addr: 0x0000,
            data_len: 0,
            data: core::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Callback handler result
pub enum CallbackHandlerResult {
    /// Everything OK
    OK = 0x00,
    /// Read timeout
    ReadTimeout = 0x02,
    /// Write timeout
    WriteTimeout = 0x03,
    /// Internal API error
    APIError = 0x04,
    /// Callback already exists. Cannot register a new one
    CallbackAlreadyExists = 0x05,
}

impl From<CallbackHandlerResult> for ChannelError {
    fn from(x: CallbackHandlerResult) -> Self {
        match x {
            CallbackHandlerResult::OK => {
                panic!("Attempted to convert OK HandlerResult into an error!?")
            }
            CallbackHandlerResult::ReadTimeout => ChannelError::ReadTimeout,
            CallbackHandlerResult::WriteTimeout => ChannelError::WriteTimeout,
            CallbackHandlerResult::CallbackAlreadyExists => ChannelError::HardwareError(
                HardwareError::APIError {
                    code: 99,
                    desc: "Callback already exists".into(),
                }
            ),
            CallbackHandlerResult::APIError => ChannelError::HardwareError(
                HardwareError::APIError {
                    code: 99,
                    desc: "Unknown error".into(),
                }
            )
        }
    }
}

#[repr(C)]
#[derive(Clone)]
#[allow(missing_debug_implementations)]
/// Callback handler for base channel to allow access via FFI
pub struct BaseChannelCallbackHandler {
    /// Callback when [BaseChannel::open] is called
    pub open_callback: extern "C" fn() -> CallbackHandlerResult,
    /// Callback when [BaseChannel::close] is called
    pub close_callback: extern "C" fn() -> CallbackHandlerResult,
    /// Callback when [BaseChannel::write_bytes] is called
    pub write_bytes_callback: extern "C" fn(
        write_payload: CallbackPayload,
        write_timeout_ms: u32,
    ) -> CallbackHandlerResult,
    /// Callback when [BaseChannel::read_bytes] is called
    pub read_bytes_callback: extern "C" fn(
        read_payload: &mut CallbackPayload,
        read_timeout_ms: u32,
    ) -> CallbackHandlerResult,
    /// Callback when [BaseChannel::clear_tx_buffer] is called
    pub clear_tx_callback: extern "C" fn() -> CallbackHandlerResult,
    /// Callback when [BaseChannel::clear_rx_buffer] is called
    pub clear_rx_callback: extern "C" fn() -> CallbackHandlerResult,
    /// Callback when [BaseChannel::set_ids] is called
    pub set_ids_callback: extern "C" fn(send: u32, recv: u32) -> CallbackHandlerResult,
}

impl PayloadChannel for BaseChannelCallbackHandler {
    fn open(&mut self) -> ChannelResult<()> {
        match (self.open_callback)() {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into()),
        }
    }

    fn close(&mut self) -> ChannelResult<()> {
        match (self.close_callback)() {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into()),
        }
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        match (self.set_ids_callback)(send, recv) {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into()),
        }
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        let mut p = CallbackPayload::default();
        match (self.read_bytes_callback)(&mut p, timeout_ms) {
            CallbackHandlerResult::OK => Ok(unsafe {
                Vec::from_raw_parts(p.data as *mut u8, p.data_len as usize, p.data_len as usize)
            }),
            x => Err(x.into()),
        }
    }

    fn write_bytes(&mut self, addr: u32, buffer: &[u8], timeout_ms: u32) -> ChannelResult<()> {
        let p = CallbackPayload {
            addr,
            data_len: buffer.len() as u32,
            data: buffer.as_ptr(),
        };

        match (self.write_bytes_callback)(p, timeout_ms) {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into()),
        }
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        match (self.clear_rx_callback)() {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into()),
        }
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        match (self.clear_tx_callback)() {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into()),
        }
    }
}

#[repr(C)]
#[derive(Clone)]
/// Callback handler for [IsoTPChannel]
pub struct IsoTpChannelCallbackHandler {
    /// Base handler
    pub base: BaseChannelCallbackHandler,
    /// Callback when [IsoTPChannel::set_iso_tp_cfg] is called
    pub set_iso_tp_cfg_callback: extern "C" fn(cfg: IsoTPSettings) -> CallbackHandlerResult,
}

impl IsoTPChannel for IsoTpChannelCallbackHandler {
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()> {
        match (self.set_iso_tp_cfg_callback)(cfg) {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into()),
        }
    }
}

impl PayloadChannel for IsoTpChannelCallbackHandler {
    fn open(&mut self) -> ChannelResult<()> {
        self.base.open()
    }

    fn close(&mut self) -> ChannelResult<()> {
        self.base.close()
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        self.base.set_ids(send, recv)
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        self.base.read_bytes(timeout_ms)
    }

    fn write_bytes(&mut self, addr: u32, buffer: &[u8], timeout_ms: u32) -> ChannelResult<()> {
        self.base.write_bytes(addr, buffer, timeout_ms)
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        self.base.clear_rx_buffer()
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        self.base.clear_tx_buffer()
    }
}

static mut ISO_TP_HANDLER: Option<IsoTpChannelCallbackHandler> = None;
static mut UDS_SERVER: Option<UdsDiagnosticServer> = None;

static mut ECU_ERROR: u8 = 0x00;

/// Register an ISO-TP callback
#[no_mangle]

pub extern "C" fn register_isotp_callback(cb: IsoTpChannelCallbackHandler) {
    unsafe { ISO_TP_HANDLER = Some(cb) }
}

/// Delete an ISO-TP callback
#[no_mangle]
pub extern "C" fn destroy_isotp_callback() {
    unsafe { ISO_TP_HANDLER = None }
}

// DIAG SERVERS

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// FFI Diagnostic server response codes
pub enum DiagServerResult {
    /// Operation OK
    OK = 0,
    /// Operation not supported by diagnostic server
    NotSupported = 1,
    /// ECU Responded with no data
    EmptyResponse = 2,
    /// ECU Responded with incorrect SID
    WrongMessage = 3,
    /// Internal diagnostic server is not running. Must have encountered a critical error
    ServerNotRunning = 4,
    /// ECU Response was of invalid length
    InvalidResponseLength = 5,
    /// No Callback handler registered
    NoHandler = 6,
    /// Diagnostic server is already running, cannot create a new one
    ServerAlreadyRunning = 7,
    /// No diagnostic server to register the request. Call
    NoDiagnosticServer = 8,
    /// Parameter provided to a subfunction was invalid
    ParameterInvalid = 9,
    HardwareError = 10,
    /// ECU responded with an error, call [get_ecu_error_code]
    /// to retrieve the NRC from the ECU
    ECUError = 98,
    /// Callback handler error
    HandlerError = 99,
    /// Function not completed in code (Will be removed in Version 1.0)
    Todo = 100,
}

impl From<DiagError> for DiagServerResult {
    fn from(x: DiagError) -> Self {
        match x {
            DiagError::NotSupported => DiagServerResult::NotSupported,
            DiagError::ECUError {code, def} => {
                unsafe { ECU_ERROR = code };
                DiagServerResult::ECUError
            }
            DiagError::EmptyResponse => DiagServerResult::EmptyResponse,
            DiagError::WrongMessage => DiagServerResult::WrongMessage,
            DiagError::ServerNotRunning => DiagServerResult::ServerNotRunning,
            DiagError::InvalidResponseLength => DiagServerResult::InvalidResponseLength,
            DiagError::NotImplemented(_) => DiagServerResult::Todo,
            DiagError::ChannelError(_) => DiagServerResult::HandlerError,
            DiagError::ParameterInvalid => DiagServerResult::ParameterInvalid,
            DiagError::HardwareError(_) => DiagServerResult::HardwareError,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
/// Payload to send to the UDS server
pub struct UdsPayload {
    /// Service ID
    pub sid: UDSCommand,
    /// Argument length
    pub args_len: u32,
    /// Pointer to arguments array
    pub args_ptr: *mut u8,
}

/// Gets the last ECU negative response code

#[no_mangle]
pub extern "C" fn get_ecu_error_code() -> u8 {
    unsafe { ECU_ERROR }
}

/// Creates a new UDS diagnostic server using an ISO-TP callback handler
#[no_mangle]
pub extern "C" fn create_uds_server_over_isotp(
    settings: UdsServerOptions,
    iso_tp_opts: IsoTPSettings,
) -> DiagServerResult {
    if unsafe { ISO_TP_HANDLER.is_none() } {
        return DiagServerResult::NoHandler;
    }
    if unsafe { UDS_SERVER.is_some() } {
        return DiagServerResult::ServerAlreadyRunning;
    }

    let channel = unsafe { ISO_TP_HANDLER.clone().unwrap() };
    let server =
        UdsDiagnosticServer::new_over_iso_tp(settings, channel, iso_tp_opts, UdsVoidHandler);

    match server {
        Ok(s) => {
            unsafe { UDS_SERVER = Some(s) }
            DiagServerResult::OK
        }
        Err(e) => e.into(),
    }
}

/// Sends a payload to the UDS server, attempts to get the ECUs response
///
/// ## Parameters
/// * payload - Payload to send to the ECU. If the ECU responds OK, then this payload
/// will be replaced by the ECUs response
///
/// * response_require - If set to false, no response will be read from the ECU.
///
/// ## Notes
///
/// Due to restrictions, the payload SID in the response message will match the original SID,
/// rather than SID + 0x40.
///
/// ## Returns
/// If a response is required, and it completes successfully, then the returned value
/// will have a new pointer set for args_ptr. **IMPORTANT**. It is up to the caller
/// of this function to deallocate this pointer after using it. The rust library will
/// have nothing to do with it once it is returned
#[no_mangle]
pub extern "C" fn send_payload_uds(
    payload: &mut UdsPayload,
    response_require: bool,
) -> DiagServerResult {
    if unsafe { UDS_SERVER.is_none() } {
        return DiagServerResult::NoDiagnosticServer;
    }

    match unsafe { UDS_SERVER.as_mut() } {
        Some(server) => {
            if response_require {
                match server.execute_command_with_response(payload.sid, unsafe {
                    &core::slice::from_raw_parts(payload.args_ptr, payload.args_len as usize)
                }) {
                    Ok(mut resp) => {
                        payload.sid = (resp[0] - 0x40).into();
                        let len = resp.len() as u32;
                        let resp_ptr = resp.as_mut_ptr();
                        core::mem::forget(resp); // Forget the response array, its up to the caller to deallocate this
                        payload.args_len = len;
                        payload.args_ptr = resp_ptr;
                        DiagServerResult::OK
                    }
                    Err(e) => e.into(),
                }
            } else {
                match server.execute_command(payload.sid, unsafe {
                    &core::slice::from_raw_parts(payload.args_ptr, payload.args_len as usize)
                }) {
                    Ok(_) => DiagServerResult::OK,
                    Err(e) => e.into(),
                }
            }
        }
        None => DiagServerResult::NoDiagnosticServer,
    }
}

/// Destroys an existing UDS server
#[no_mangle]
pub extern "C" fn destroy_uds_server() {
    unsafe { UDS_SERVER = None }
}
