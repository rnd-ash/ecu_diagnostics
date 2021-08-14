#![deny(missing_docs, missing_debug_implementations)]

//! A crate which provides the most common ECU diagnostic protocols used by modern ECUs in vehicles.
//!
//! ## Protocol support
//!
//! This crate provides the 3 most widely used diagnostic protocols used by modern ECUs from 2000 onwards
//!
//! ### On-board diagnostics (OBD2)
//! ISO9141 - OBD2 is a legal requirement on all vehicles produced from 2002, allowing for
//! reading of sensor data, reading and clearing standard DTCs, and reading basic vehicle information.
//! OBD2 is designed to be safe and simple, and does not write data to the ECU.
//!
//!
//!
//! ### Keyword protocol 2000 (KWP2000)
//! ISO14230 - KWP2000 is a advanced diagnostic protocol utilized by many vehicle manufacturers from 2000-2006 (Superseded by UDS).
//! Unlike OBD2, KWP2000 allows for much more complex operations, which could potentially cause damage to a vehicle if used incorrectly.  
//! A few examples of features allowed by KWP2000 are
//! * ECU flashing
//! * Clearing and reading of permanent DTCs
//! * Manipulation of ECU communication parameters
//! * Low level manipulation of ECU's EEPROM or RAM
//! * Gateway access in vehicles which have them
//!
//! The specification implemented in this crate is v2.2, dated 05-08-2002
//!
//! ### Unified diagnostic services (UDS)
//! ISO14429 - UDS is an advanced diagnostic protocol utilized by almost all vehicle manufacturers from 2006 onwards. Like KWP2000,
//! this protocol allows for reading/writing directly to the ECU, and should therefore be used with caution.
//!
//! The specification implemented in this crate is the second edition, dated 01-12-2006.
//!
//! ## Usage
//! In order to utilize any of the diagnostic servers, you will need
//! to implement the [channel::BaseChannel] trait, which allows for the diagnostic servers
//! to send and receive data to/from the ECU, regardless of the transport layer used.
//!

use channel::ChannelError;
use hardware::HardwareError;

pub mod channel;
pub mod dtc;
pub mod hardware;
pub mod kwp2000;
pub mod obd2;
pub mod uds;

mod helpers;

/// Diagnostic server result
pub type DiagServerResult<T> = Result<T, DiagError>;

#[derive(Debug)]
/// Diagnostic server error
pub enum DiagError {
    /// The Diagnostic server does not support the request
    NotSupported,
    /// Diagnostic error code from the ECU itself
    ECUError(u8),
    /// Response empty
    EmptyResponse,
    /// ECU Responded but send a message that wasn't a reply for the sent message
    WrongMessage,
    /// Diagnostic server terminated!?
    ServerNotRunning,
    /// ECU Responded with a message, but the length was incorrect
    InvalidResponseLength,
    /// A parameter given to the function is invalid. Check the function's documentation
    /// for more information
    ParameterInvalid,
    /// Error with underlying communication channel
    ChannelError(ChannelError),
    /// Denotes a TODO action (Non-implemented function stub)
    /// This will be removed in Version 1
    NotImplemented(String),
    /// Device hardware error
    HardwareError(HardwareError),
}

impl std::fmt::Display for DiagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DiagError::NotSupported => write!(f, "request not supported"),
            DiagError::ECUError(err) => write!(f, "ECU error 0x{:02X}", err),
            DiagError::EmptyResponse => write!(f, "ECU provided an empty response"),
            DiagError::WrongMessage => write!(f, "ECU response message did not match request"),
            DiagError::ServerNotRunning => write!(f, "diagnostic server not running"),
            DiagError::ParameterInvalid => write!(f, "a parameter provided was invalid"),
            DiagError::InvalidResponseLength => {
                write!(f, "ECU response message was of invalid length")
            }
            DiagError::ChannelError(err) => write!(f, "underlying channel error: {}", err),
            DiagError::NotImplemented(s) => {
                write!(f, "server encountered an unimplemented function: {}", s)
            }
            &DiagError::HardwareError(e) => write!(f, "Hardware error: {}", e),
        }
    }
}

impl std::error::Error for DiagError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            DiagError::ChannelError(e) => Some(e),
            DiagError::HardwareError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ChannelError> for DiagError {
    fn from(x: ChannelError) -> Self {
        Self::ChannelError(x)
    }
}

impl From<HardwareError> for DiagError {
    fn from(x: HardwareError) -> Self {
        Self::HardwareError(x)
    }
}

#[derive(Debug)]
/// Diagnostic server event
pub enum ServerEvent<'a, SessionState, RequestType> {
    /// The diagnostic server encountered an unrecoverable critical error
    CriticalError {
        /// Text description of the error
        desc: String,
    },
    /// The diagnostic server has started
    ServerStart,
    /// The diagnostic server has terminated
    ServerExit,
    /// The diagnostic server has changed session state
    DiagModeChange {
        /// Old session state
        old: SessionState,
        /// New session state
        new: SessionState,
    },
    /// Received a request to send a payload to the ECU
    IncomingEvent(&'a RequestType),
    /// Response from the ECU
    OutgoingEvent(&'a DiagServerResult<RequestType>),
    /// An error occurred whilst transmitting tester present message
    /// To the ECU. This might mean that the ECU has exited its session state,
    /// and a non-default session state should be re-initialized
    TesterPresentError(DiagError),
    /// Error occurred whilst trying to terminate the server's channel interface
    /// when the diagnostic server exited.
    InterfaceCloseOnExitError(ChannelError),
}

unsafe impl<'a, SessionType, RequestType> Send for ServerEvent<'a, SessionType, RequestType> {}
unsafe impl<'a, SessionType, RequestType> Sync for ServerEvent<'a, SessionType, RequestType> {}

/// Handler for when [ServerEvent] get broadcast by the diagnostic servers background thread
pub trait ServerEventHandler<SessionState, RequestType>: Send + Sync {
    /// Handle incoming server events
    fn on_event(&mut self, e: ServerEvent<SessionState, RequestType>);
}

/// Basic diagnostic server settings
pub trait BaseServerSettings {
    /// Gets the write timeout for sending messages to the servers channel
    fn get_write_timeout_ms(&self) -> u32;
    /// Gets the read timeout for reading response messages from the servers channel
    fn get_read_timeout_ms(&self) -> u32;
}

/// Basic diagnostic server payload
pub trait BaseServerPayload {
    /// Gets the payload portion of the diagnostic message (Not including the SID)
    fn get_payload(&self) -> &[u8];
    /// Gets the SID (Service ID) byte from the payload
    fn get_sid_byte(&self) -> u8;
    /// Gets the entire message as a byte array. This is what is sent to the ECU
    fn to_bytes(&self) -> &[u8];
    /// Boolean indicating if the diagnostic server should poll the ECU for a response after sending the payload
    fn requires_response(&self) -> bool;
}
