#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unused_imports,
    unused_import_braces,
    unused_qualifications
)]

//! A crate which provides the most common ECU diagnostic protocols used by modern ECUs in vehicles,
//! as well as common hardware APIs for accessing and using diagnostic adapters
//!
//! ## ECU Diagnostic protocol support
//!
//! This crate provides the 3 most widely used diagnostic protocols used by modern ECUs from 2000 onwards
//!
//! ### On-board diagnostics (OBD2)
//! ISO9141 - OBD2 is a legal requirement on all vehicles produced from 2002, allowing for
//! reading of sensor data, reading and clearing standard DTCs, and reading basic vehicle information.
//! OBD2 is designed to be safe and simple, and does not write data to the ECU.
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
//! The specification implemented in this crate is v2.2, dated 05-08-2002.
//!
//!
//! ### Unified diagnostic services (UDS)
//! ISO14429 - UDS is an advanced diagnostic protocol utilized by almost all vehicle manufacturers from 2006 onwards. Like KWP2000,
//! this protocol allows for reading/writing directly to the ECU, and should therefore be used with caution.
//!
//! The specification implemented in this crate is the second edition, dated 01-12-2006.
//!
//! ## Hardware support (VCIs)
//!
//! This crate provides support for the following VCI adapters and hardware protocols, as well as a convenient interface
//! for making your own adapter API for customized hardware
//!
//! ### SocketCAN (Linux only)
//! This crate provides support for socketCAN compatible adapters, for utilizing both ISO-TP and regular CAN communication
//!
//! ### SAE J2534-2
//!
//! SAE J2534 (AKA Passthru) is a VCI adapter protocol which allows a VCI to communicate with a vehicle using multiple various
//! different network types, including CAN, ISO-TP, J1850, ISO9141 and SCI.
//!
//! NOTE: Although the J2534 API is officially only for Windows, it will also compile for UNIX and OSX operating
//! systems, due to the unofficial porting of the API in the [Macchina-J2534 project](https://github.com/rnd-ash/Macchina-J2534)
//!

use channel::ChannelError;
use hardware::HardwareError;

pub mod channel;
pub mod dtc;
pub mod dynamic_diag;
pub mod hardware;
pub mod kwp2000;
pub mod obd2;
pub mod uds;

pub use automotive_diag::ByteWrapper::*;

/// Diagnostic server result
pub type DiagServerResult<T> = Result<T, DiagError>;

#[derive(Debug)]
/// Diagnostic server error
pub enum DiagError {
    /// The Diagnostic server does not support the request
    NotSupported,
    /// Diagnostic error code from the ECU itself
    ECUError {
        /// Raw Negative response code from ECU
        code: u8,
        /// Negative response code definition according to protocol
        def: Option<String>,
    },
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
    /// ECU Param ID did not match the request, but the Service ID was correct
    MismatchedResponse(String),
}

impl std::fmt::Display for DiagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DiagError::NotSupported => write!(f, "request not supported"),
            DiagError::ECUError { code, def } => {
                if let Some(d) = def {
                    write!(f, "ECU error 0x{:02X} ({})", code, d)
                } else {
                    write!(f, "ECU error 0x{:02X}", code)
                }
            }
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
            DiagError::HardwareError(e) => write!(f, "Hardware error: {}", e),
            DiagError::MismatchedResponse(e) => write!(f, "Param mismatched response: {}", e),
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
