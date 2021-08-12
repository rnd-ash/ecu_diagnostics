//! The hardware module contains simplified API's
//! for interacting with common hardware that can be used for either Bench setups or OBD2 adapters

pub mod passthru;

use crate::channel::{CanChannel, IsoTPChannel};

/// Hardware API result
pub type HardwareResult<T> = std::result::Result<T, HardwareError>;

/// The hardware trait defines a functions supported by all adapters
/// and also functions used to convert API implementations into [super::channel::IsoTPChannel]
pub trait Hardware<'a> {
    /// Gets the devices ISO TP Channel
    /// This channel will live as long as lifetime 'a. Upon being dropped,
    /// the channel will automatically be dropped and closed
    fn create_iso_tp_channel(&'a mut self) -> HardwareResult<Box<&'a dyn IsoTPChannel>>;
    /// Gets the devices CAN Channel.
    /// This channel will live as long as lifetime 'a. Upon being dropped,
    /// the channel will automatically be dropped and closed
    fn create_can_channel(&'a mut self) -> HardwareResult<Box<&'a dyn CanChannel>>;

    /// Tries to read battery voltage from Pin 16 of an OBD port (+12V).
    /// This is mainly used by diagnostic adapters, and is purely optional
    /// Should the adapter not support this feature, [std::option::Option::None] is returned
    fn read_battery_voltage(&mut self) -> Option<f32>;

    /// Returns a list of hardware capabilities
    fn get_capabilities(&self) -> &HardwareCapabilities;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Device hardware info used by [HardwareScanner]
pub struct HardwareInfo {
    /// Name of the hardware
    pub name: String,
    /// Vendor of the hardware
    pub vendor: String,
    /// Listed capabilities of the hardware
    pub capabilities: HardwareCapabilities
}

/// Trait for scanning for supported adapters, given an API
pub trait HardwareScanner<T: Hardware<'static>> {
    /// Lists all scanned devices
    fn list_devices(&self) -> Vec<HardwareInfo>;
    /// Tries to open a device by a specific index.
    fn open_device_by_index(&self, idx: usize) -> HardwareResult<T>;
    /// Tries to open a device given the devices name
    fn open_device_by_name(&self, name: &str) -> HardwareResult<T>;
}

#[derive(Debug)]
/// Represents error that can be returned by Hardware API
pub enum HardwareError {
    /// Low level driver error
    APIError {
        /// API Error code
        code: u32, 
        /// API Error description
        desc: String
    },
    /// Indicates a conflict of channel, An example would be having an ISOTP channel open and
    /// then also trying to open a CAN Channel at the same time. This cannot occur
    /// as both use the same physical data layer and thus hardware
    ConflictingChannel,
    /// Indicates a channel type is not supported by the API
    ChannelNotSupported,
    /// Hardware not found
    DeviceNotFound,
    /// Function called on device that has not been opened
    DeviceNotOpen,
    /// Lib loading error
    LibLoadError(libloading::Error)
}
impl From<libloading::Error> for HardwareError {
    fn from(err: libloading::Error) -> Self {
        Self::LibLoadError(err)
    }
}

impl std::fmt::Display for HardwareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            HardwareError::APIError { code, desc } => write!(f, "Hardware API Error. Code {}, Description: {}", code, desc),
            HardwareError::ConflictingChannel => write!(f, "Conflicting communication channel already open"),
            HardwareError::ChannelNotSupported => write!(f, "Channel type is not supported by hardware"),
            HardwareError::DeviceNotFound => write!(f, "Device not found"),
            HardwareError::DeviceNotOpen => write!(f, "Hardware device not open"),
            HardwareError::LibLoadError(e) => write!(f, "LibLoading error: {}", e),
        }
    }
}

impl std::error::Error for HardwareError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            &HardwareError::LibLoadError(l) => Some(l),
            _ => None
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
    /// Supports IP protocols (DOIP)
    pub ip: bool
}