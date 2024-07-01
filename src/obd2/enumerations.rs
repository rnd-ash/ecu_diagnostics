//! Enumeration data for OBD services 01 and 02

use automotive_diag::obd2::{
    CommandedSecondaryAirStatusByte, FuelSystemStatusByte, FuelTypeCodingByte, ObdStandardByte,
};
use automotive_diag::ByteWrapper::Standard;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// OBD enumeration type wrapper
pub enum ObdEnumValue {
    /// Fuel system status
    FuelSystemStatus(FuelSystemStatusByte),
    /// Commanded secondary air status
    CommandedAirStatus(CommandedSecondaryAirStatusByte),
    /// OBD standard
    ObdStandard(ObdStandardByte),
    /// Vehicle fuel type
    FuelType(FuelTypeCodingByte),
}

impl From<ObdEnumValue> for u32 {
    fn from(x: ObdEnumValue) -> u32 {
        let u: u8 = match x {
            ObdEnumValue::FuelSystemStatus(x) => x.into(),
            ObdEnumValue::CommandedAirStatus(x) => x.into(),
            ObdEnumValue::ObdStandard(x) => x.into(),
            ObdEnumValue::FuelType(x) => x.into(),
        };
        u as u32
    }
}

impl Display for ObdEnumValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ObdEnumValue::FuelSystemStatus(x) => match x {
                Standard(v) => Display::fmt(&v, f),
                _ => f.write_fmt(format_args!("Extended({:#02X})", u32::from(*self))),
            },
            ObdEnumValue::CommandedAirStatus(x) => match x {
                Standard(v) => Display::fmt(&v, f),
                _ => f.write_fmt(format_args!("Extended({:#02X})", u32::from(*self))),
            },
            ObdEnumValue::ObdStandard(x) => match x {
                Standard(v) => Display::fmt(&v, f),
                _ => f.write_fmt(format_args!("Extended({:#02X})", u32::from(*self))),
            },
            ObdEnumValue::FuelType(x) => match x {
                Standard(v) => Display::fmt(&v, f),
                _ => f.write_fmt(format_args!("Extended({:#02X})", u32::from(*self))),
            },
        }
    }
}
