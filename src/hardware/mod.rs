//! The hardware module contains simplified API's and abstraction layers
//! for interacting with common hardware that can be used for either Bench setups or OBD2 adapters
//! in order to communicate with vehicle ECUs

mod dpdu;

#[cfg(feature = "passthru")]
pub mod passthru; // Not finished at all yet, hide from the crate

#[cfg(feature = "passthru")]
use std::sync::Arc;

#[cfg(all(feature = "socketcan", target_os = "linux"))]
pub mod socketcan;

use crate::channel::{CanChannel, IsoTPChannel};

/// Hardware API result
pub type HardwareResult<T> = Result<T, HardwareError>;

/// The hardware trait defines functions supported by all adapter types,
/// as well as functions that can create abstracted communication channels
/// that can be used in diagnostic servers
pub trait Hardware: Clone {
    /// Creates an ISO-TP channel on the devices.
    /// This channel will live for as long as the hardware trait. Upon being dropped,
    /// the channel will automatically be closed, if it has been opened.
    fn create_iso_tp_channel(&mut self) -> HardwareResult<Box<dyn IsoTPChannel>>;

    /// Creates a CAN Channel on the devices.
    /// This channel will live for as long as the hardware trait. Upon being dropped,
    /// the channel will automatically be closed, if it has been opened.
    fn create_can_channel(&mut self) -> HardwareResult<Box<dyn CanChannel>>;

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

    /// Returns if the hardware is currently connected
    fn is_connected(&self) -> bool;
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
    fn open_device_by_index(&self, idx: usize) -> HardwareResult<T>;
    /// Tries to open a device given the devices name
    fn open_device_by_name(&self, name: &str) -> HardwareResult<T>;
}

#[derive(Clone, Debug, thiserror::Error)]
/// Represents error that can be returned by Hardware API
pub enum HardwareError {
    /// Low level device driver error
    #[error("Device library API error. Code {code}, Description: '{desc}'")]
    APIError {
        /// API Error code
        code: u32,
        /// API Error description
        desc: String,
    },
    /// Indicates that a conflicting channel type was opened on a device which does not
    /// support multiple channels of the same underlying network to be open at once.
    #[error("Channel type conflicts with an already open channel")]
    ConflictingChannel,
    /// Indicates a channel type is not supported by the API
    #[error("Channel type not supported on this hardware")]
    ChannelNotSupported,
    /// Hardware not found
    #[error("Device not found")]
    DeviceNotFound,
    /// Function called on device that has not yet been opened
    #[error("Device was not opened")]
    DeviceNotOpen,

    /// Lib loading error
    #[cfg(feature = "passthru")]
    #[error("Device API library load error")]
    LibLoadError(
        #[from]
        #[source]
        Arc<libloading::Error>,
    ),
}

#[cfg(feature = "passthru")]
impl From<libloading::Error> for HardwareError {
    fn from(err: libloading::Error) -> Self {
        Self::LibLoadError(Arc::new(err))
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
