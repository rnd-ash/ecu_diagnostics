//! The hardware module contains simplified API's and abstraction layers
//! for interacting with common hardware that can be used for either Bench setups or OBD2 adapters
//! in order to communicate with vehicle ECUs

#[cfg(feature = "passthru")]
pub mod passthru;

#[cfg(all(feature = "dpdu"))]
pub mod dpdu;

#[cfg(all(feature = "socketcan", unix))]
pub mod socketcan;

use std::sync::{Arc, Mutex};

use crate::channel::{CanChannel, IsoTPChannel};

/// Hardware API result
pub type HardwareResult<T> = Result<T, HardwareError>;

/// The hardware trait defines functions supported by all adapter types,
/// as well as functions that can create abstracted communication channels
/// that can be used in diagnostic servers
pub trait Hardware {
    /// Creates an ISO-TP channel on the devices.
    /// This channel will live for as long as the hardware trait. Upon being dropped,
    /// the channel will automatically be closed, if it has been opened.
    fn create_iso_tp_channel(this: Arc<Mutex<Self>>) -> HardwareResult<Box<dyn IsoTPChannel>>;

    /// Creates a CAN Channel on the devices.
    /// This channel will live for as long as the hardware trait. Upon being dropped,
    /// the channel will automatically be closed, if it has been opened.
    fn create_can_channel(this: Arc<Mutex<Self>>) -> HardwareResult<Box<dyn CanChannel>>;

    /// Returns true if the ISO-TP channel is current open and in use
    fn is_iso_tp_channel_open(&self) -> bool;

    /// Returns true if the CAN channel is currently open and in use
    fn is_can_channel_open(&self) -> bool;

    /// Tries to read battery voltage from Pin 16 of an OBD port (+12V).
    /// This is mainly used by diagnostic adapters, and is purely optional
    /// Should the adapter not support this feature, [std::option::Option::None] is returned
    fn read_battery_voltage(&mut self) -> Option<f32>;

    /// Tries to read battery voltage from the igntion pin on the OBD2 port. A reading
    /// would indicate ignition is on in the vehicle.
    /// This is mainly used by diagnostic adapters, and is purely optional
    /// Should the adapter not support this feature, [std::option::Option::None] is returned
    fn read_ignition_voltage(&mut self) -> Option<f32>;

    /// Returns the information of the hardware
    fn get_info(&self) -> &HardwareInfo;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Device hardware info used by [HardwareScanner]
pub struct HardwareInfo {
    /// Name of the hardware
    pub name: String,
    /// Optional vendor of the hardware
    pub vendor: Option<String>,
    /// Optional version of the firmware running on the adapter / device
    pub device_fw_version: Option<String>,
    /// Optional API standard the device conforms to
    pub api_version: Option<String>,
    /// Optional library (Dll/So/Dynlib) version used
    pub library_version: Option<String>,
    /// Optional file location of the library used
    pub library_location: Option<String>,
    /// Listed capabilities of the hardware
    pub capabilities: HardwareCapabilities,
}

/// Trait for scanning hardware on a system which can be used
/// to diagnose ECUs
pub trait HardwareScanner<T: Hardware> {
    /// Lists all scanned devices. This does not necessarily
    /// mean that the hardware can be used, just that the system
    /// known it exists.
    fn list_devices(&self) -> Vec<HardwareInfo>;
    /// Tries to open a device by a specific index from the [HardwareScanner::list_devices] function.
    fn open_device_by_index(&self, idx: usize) -> HardwareResult<Arc<Mutex<T>>>;
    /// Tries to open a device given the devices name
    fn open_device_by_name(&self, name: &str) -> HardwareResult<Arc<Mutex<T>>>;
}

#[derive(Debug)]
/// Represents error that can be returned by Hardware API
pub enum HardwareError {
    /// Low level device driver error
    APIError {
        /// API Error code
        code: u32,
        /// API Error description
        desc: String,
    },
    /// Indicates that a conflicting channel type was opened on a device which does not
    /// support multiple channels of the same underlying network to be open at once.
    ConflictingChannel,
    /// Indicates a channel type is not supported by the API
    ChannelNotSupported,
    /// Hardware not found
    DeviceNotFound,
    /// Function called on device that has not yet been opened
    DeviceNotOpen,

    /// Lib loading error
    #[cfg(feature = "passthru")]
    LibLoadError(libloading::Error),
}
#[cfg(feature = "passthru")]
impl From<libloading::Error> for HardwareError {
    fn from(err: libloading::Error) -> Self {
        Self::LibLoadError(err)
    }
}

impl std::fmt::Display for HardwareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            HardwareError::APIError { code, desc } => write!(
                f,
                "Hardware API Error. Code {}, Description: {}",
                code, desc
            ),
            HardwareError::ConflictingChannel => {
                write!(f, "Conflicting communication channel already open")
            }
            HardwareError::ChannelNotSupported => {
                write!(f, "Channel type is not supported by hardware")
            }
            HardwareError::DeviceNotFound => write!(f, "Device not found"),
            HardwareError::DeviceNotOpen => write!(f, "Hardware device not open"),
            #[cfg(feature = "passthru")]
            HardwareError::LibLoadError(e) => write!(f, "LibLoading error: {}", e),
        }
    }
}

impl std::error::Error for HardwareError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            #[cfg(feature = "passthru")]
            HardwareError::LibLoadError(l) => Some(l),
            _ => None,
        }
    }
}

/// Contains details about what communication protocols
/// are supported by the physical hardware
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HardwareCapabilities {
    /// Supports ISO-TP
    pub iso_tp: bool,
    /// Supports CANBUS
    pub can: bool,
    /// Supports standard Kline OBD (ISO9141)
    pub kline: bool,
    /// Supports KWP2000 over Kline (ISO14230)
    pub kline_kwp: bool,
    /// Supports J1850 VPW and J180 PWM
    pub sae_j1850: bool,
    /// Supports Chryslers serial communication interface
    pub sci: bool,
    /// Supports IP protocols (Diagnostic Over IP)
    pub ip: bool,
}
