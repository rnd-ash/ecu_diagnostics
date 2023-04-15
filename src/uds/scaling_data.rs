///! Functions and data for ReadScalingDataById UDS Service

/// FIXME: Use ScalingExtension instead
/// Note: `#[deprecated]` doesn't work here due to https://github.com/rust-lang/rust/issues/30827
pub use auto_uds::uds::ScalingExtension as ScalingByteExtension;

/// FIXME: Use ScalingType instead
/// Note: `#[deprecated]` doesn't work here due to https://github.com/rust-lang/rust/issues/30827
pub use auto_uds::uds::ScalingType as ScalingByteHigh;

pub use auto_uds::uds::{ScalingExtension, ScalingType};

/// Represents Scaling data structure returned from ECU
#[derive(Debug, Clone)]
pub struct ScalingData {
    x: f32,
    c0: f32,
    c1: f32,
    c2: f32,
    mapping_byte: u8,
    byte_ext: Vec<ScalingExtension>,
}

impl ScalingData {
    /// Creates a new scaling data structure
    pub(crate) fn new(
        x: i32,
        c0: i32,
        c1: i32,
        c2: i32,
        mapping_byte: u8,
        byte_ext: &[ScalingExtension],
    ) -> Self {
        Self {
            x: x as f32,
            c0: c0 as f32,
            c1: c1 as f32,
            c2: c2 as f32,
            mapping_byte,
            byte_ext: byte_ext.to_vec(),
        }
    }

    /// Returns the list of scaling data presentation of the scaling data.
    /// Note that there can be more than one! (EG: Having a prefix and postfix scaling byte)
    pub fn get_scaling_byte(&self) -> &[ScalingExtension] {
        &self.byte_ext
    }

    /// Returns a converted value from raw.
    /// If the conversion formula falls under VMS (Vehicle manufacture specific), then None is returned.
    pub fn get_mapping_from_raw(&self) -> Option<f32> {
        let c0 = self.c0;
        let c1 = self.c1;
        let c2 = self.c2;
        let x = self.x;
        match self.mapping_byte {
            0x00 => Some(c0 * x + c1),
            0x01 => Some(c0 * (x + c1)),
            0x02 => Some(c0 / (x + c1) + c2),
            0x03 => Some(x / (c0 + c1)),
            0x04 => Some((x + c0) / c1),
            0x05 => Some((x + c0) / c1 + c2),
            0x06 => Some(c0 * x),
            0x07 => Some(x / c0),
            0x08 => Some(x + c0),
            0x09 => Some(x * c0 / c1),
            _ => None, // VMS or reserved
        }
    }
}
