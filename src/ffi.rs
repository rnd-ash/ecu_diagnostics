//! FFI bindings for ECU_Diagnostics
//!
//! IMPORTANT. Access to the FFI bindings should be done on one thread! No multi-thread support


use std::intrinsics::transmute;

use crate::{DiagError, channel::{BaseChannel, ChannelError, IsoTPChannel, IsoTPSettings}, uds::{UDSCommand, UdsDiagnosticServer, UdsServerOptions, UdsVoidHandler}};


#[repr(C)]
#[derive(Debug)]
/// Callback handler payload
pub struct CallbackPayload {
    /// Target send address
    pub addr: u32,
    /// Data size
    pub data_len: u32,
    /// Data
    pub data: [u8; 4096]
}

impl Default for CallbackPayload {
    fn default() -> Self {
        Self {
            addr: 0x0000,
            data_len: 0,
            data: [0x00; 4096]
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
    CallbackAlreadyExists = 0x05
}

impl From<CallbackHandlerResult> for ChannelError {
    fn from(x: CallbackHandlerResult) -> Self {
        match x {
            CallbackHandlerResult::OK => panic!("Attempted to convert OK HandlerResult into an error!?"),
            CallbackHandlerResult::ReadTimeout => ChannelError::ReadTimeout,
            CallbackHandlerResult::WriteTimeout => ChannelError::WriteTimeout,
            CallbackHandlerResult::CallbackAlreadyExists => ChannelError::APIError { api_name: "NativeCallback".into(), code: 98,  desc: "Callback already registered".into() },
            CallbackHandlerResult::APIError => ChannelError::APIError { api_name: "NativeCallback".into(), code: 99, desc: "Unknown".into()  },
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
    pub write_bytes_callback: extern "C" fn(write_payload: CallbackPayload, write_timeout_ms: u32) -> CallbackHandlerResult,
    /// Callback when [BaseChannel::read_bytes] is called
    pub read_bytes_callback: extern "C" fn(read_payload: &mut CallbackPayload, read_timeout_ms: u32) -> CallbackHandlerResult,
    /// Callback when [BaseChannel::clear_tx_buffer] is called
    pub clear_tx_callback: extern "C" fn() -> CallbackHandlerResult,
    /// Callback when [BaseChannel::clear_rx_buffer] is called
    pub clear_rx_callback: extern "C" fn() -> CallbackHandlerResult,
    /// Callback when [BaseChannel::set_ids] is called
    pub set_ids_callback: extern "C" fn(send: u32, recv: u32) -> CallbackHandlerResult
}

impl BaseChannel for BaseChannelCallbackHandler {
    fn open(&mut self) -> crate::channel::ChannelResult<()> {
        match (self.open_callback)() {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into())
        }
    }

    fn close(&mut self) -> crate::channel::ChannelResult<()> {
        match (self.close_callback)() {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into())
        }
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> crate::channel::ChannelResult<()> {
        match (self.set_ids_callback)(send, recv) {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into())
        }
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> crate::channel::ChannelResult<Vec<u8>> {
        let mut p = CallbackPayload::default();
        match (self.read_bytes_callback)(&mut p, timeout_ms) {
            CallbackHandlerResult::OK => {
                Ok(p.data[0..p.data_len as usize].to_vec())
            },
            x => Err(x.into())
        }
    }

    fn write_bytes(&mut self, addr: u32, buffer: &[u8], timeout_ms: u32) -> crate::channel::ChannelResult<()> {
        let mut p = CallbackPayload {
            addr,
            data_len: buffer.len() as u32,
            data: [0; 4096],
        };
        p.data[0..buffer.len()].copy_from_slice(buffer);
        match (self.write_bytes_callback)(p, timeout_ms) {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into())
        }
    }

    fn clear_rx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        match (self.clear_rx_callback)() {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into())
        }
    }

    fn clear_tx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        match (self.clear_tx_callback)() {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into())
        }
    }
}


#[repr(C)]
#[derive(Clone)]
#[allow(missing_debug_implementations)]
/// Callback handler for [IsoTPChannel]
pub struct IsoTpChannelCallbackHandler {
    /// Base handler
    pub base: BaseChannelCallbackHandler,
    /// Callback when [IsoTPChannel::set_iso_tp_cfg] is called
    pub set_iso_tp_cfg_callback: extern "C" fn(cfg: crate::channel::IsoTPSettings) -> CallbackHandlerResult
}

impl IsoTPChannel for IsoTpChannelCallbackHandler {
    fn set_iso_tp_cfg(&mut self, cfg: crate::channel::IsoTPSettings) -> crate::channel::ChannelResult<()> {
        match (self.set_iso_tp_cfg_callback)(cfg) {
            CallbackHandlerResult::OK => Ok(()),
            x => Err(x.into())
        }
    }
}

impl BaseChannel for IsoTpChannelCallbackHandler {
    fn open(&mut self) -> crate::channel::ChannelResult<()> { self.base.open() }

    fn close(&mut self) -> crate::channel::ChannelResult<()> { self.base.close() }

    fn set_ids(&mut self, send: u32, recv: u32) -> crate::channel::ChannelResult<()> { self.base.set_ids(send, recv) }

    fn read_bytes(&mut self, timeout_ms: u32) -> crate::channel::ChannelResult<Vec<u8>> { self.base.read_bytes(timeout_ms) }

    fn write_bytes(&mut self, addr: u32, buffer: &[u8], timeout_ms: u32) -> crate::channel::ChannelResult<()> { self.base.write_bytes(addr, buffer, timeout_ms) }

    fn clear_rx_buffer(&mut self) -> crate::channel::ChannelResult<()> { self.base.clear_rx_buffer() }

    fn clear_tx_buffer(&mut self) -> crate::channel::ChannelResult<()> { self.base.clear_tx_buffer() }
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
    /// ECU responded with an error, call [get_ecu_error_code]
    /// to retrieve the NRC from the ECU
    ECUError = 98,
    /// Callback handler error
    HandlerError = 99
}

impl From<DiagError> for DiagServerResult {
    fn from(x: DiagError) -> Self {
        match x {
            DiagError::NotSupported => DiagServerResult::NotSupported,
            DiagError::ECUError(x) => {
                unsafe { ECU_ERROR = x };
                DiagServerResult::ECUError
            },
            DiagError::EmptyResponse => DiagServerResult::EmptyResponse,
            DiagError::WrongMessage => DiagServerResult::WrongMessage,
            DiagError::ServerNotRunning => DiagServerResult::ServerNotRunning,
            DiagError::InvalidResponseLength => DiagServerResult::InvalidResponseLength,
            DiagError::ChannelError(_) => DiagServerResult::HandlerError,
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
    /// Arguments
    pub args: [u8; 4095]
}

/// Gets the last ECU negative response code

#[no_mangle]
pub extern "C" fn get_ecu_error_code() -> u8 {
    unsafe { ECU_ERROR }
}

/// Creates a new UDS diagnostic server using an ISO-TP callback handler
#[no_mangle]
pub extern "C" fn create_uds_server_over_isotp(settings: UdsServerOptions, iso_tp_opts: IsoTPSettings) -> DiagServerResult {
    if unsafe { ISO_TP_HANDLER.is_none() } {
        return DiagServerResult::NoHandler;
    }
    if unsafe { UDS_SERVER.is_some() } {
        return DiagServerResult::ServerAlreadyRunning;
    }

    let channel = unsafe {ISO_TP_HANDLER.clone().unwrap() };
    let server = UdsDiagnosticServer::new_over_iso_tp(
        settings, 
        channel, 
        iso_tp_opts, 
        UdsVoidHandler
    );

    match server {
        Ok(s) => {
            unsafe { UDS_SERVER = Some(s) }
            DiagServerResult::OK
        },
        Err(e) => {
            e.into()
        }
    }
}

#[no_mangle]
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
pub extern "C" fn send_payload_uds(payload: &mut UdsPayload, response_require: bool) -> DiagServerResult {
    if unsafe { UDS_SERVER.is_none() } {
        return DiagServerResult::NoDiagnosticServer
    }

    match unsafe { UDS_SERVER.as_mut() } {
        Some(server) => {
            if response_require {
                match server.execute_command_with_response(payload.sid, &payload.args[0..payload.args_len as usize]) {
                    Ok(resp) => {
                        payload.sid = unsafe { transmute(resp[0] - 0x40) };
                        DiagServerResult::OK
                    },
                    Err(e) => e.into()
                }
            } else {
                match server.execute_command(payload.sid, &payload.args[0..payload.args_len as usize]) {
                    Ok(_) => DiagServerResult::OK,
                    Err(e) => e.into()
                }
            }
        },
        None => DiagServerResult::NoDiagnosticServer
    }
}

/// Destroys an existing UDS server
#[no_mangle]
pub extern "C" fn destroy_uds_server() {
    unsafe { UDS_SERVER = None }
}