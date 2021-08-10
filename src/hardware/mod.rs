//! The hardware module contains simplified API's
//! for interacting with common hardware that can be used for either Bench setups or OBD2 adapters

pub mod passthru;

use crate::channel::{CanChannel, IsoTPChannel};

/// The hardware trait defines a functions supported by all adapters
/// and also functions used to convert API implementations into [super::channel::IsoTPChannel]
pub trait Hardware {

    /// Converts to ISOTP channel
    fn to_iso_tp_channel(&mut self) -> Box<dyn IsoTPChannel>;
    /// Converts to a CAN Channel
    fn to_can_channel(&mut self) -> Box<dyn CanChannel>;

    /// Tries to read battery voltage from Pin 16 of an OBD port.
    /// This is mainly used by diagnostic adapters, and is purely optional
    fn read_battery_voltage(&mut self) -> Option<f32>;

}

#[derive(Debug, Clone)]
/// Represents error that can be returned by Hardware API
pub enum HardwareError {
    /// Low level driver error
    APIError {
        /// API Error code
        code: u32, 
        /// API Error description
        desc: String
    },
    /// Indicates a conflict of channel, IE: Having an ISOTP channel open and
    /// then also trying to open a CAN Channel at the same time
    ConflictingChannel,
    /// Indicates a channel type is not supported by the API
    ChannelNotSupported
}
