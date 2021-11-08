use crate::uds::ScalingByteHigh::UnsignedNumeric;

///! Functions and data for ReadScalingDataById UDS Service

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// Scaling data byte extensions
/// This enum is used to represent the following:
/// 1. Measurement units
/// 2. Format specifiers
/// 3. Unit scale prefixes
///
/// Due to this, each value specifies if it will return a Postfix or prefix.
/// Use [ScalingByteExtension::get_postfix] to return the optional postfix of the scaling byte,
/// or [ScalingByteExtension::get_prefix] to return the optional prefix of the scaling byte.
pub enum ScalingByteExtension {
    /// No unit or presentation
    NoUnit,
    /// Meter - Measure of length. Postfix: `m`
    Meter,
    /// Foot - Measure of length. Postfix: `ft`
    Foot,
    /// Inch - Measure of length. Postfix: `in`
    Inch,
    /// Yard - Measure of length. Postfix: `yd`
    Yard,
    /// English mile - Measure of length. Postfix: `mi`
    EngMile,
    /// Gram - Measure of mass. Postfix: `g`
    Gram,
    /// Metric ton - Measure of mass. Postfix: `t`
    MetricTon,
    /// Second - Measure of time. Postfix: `s`
    Second,
    /// Minute - Measure of time. Postfix: `min`
    Minute,
    /// Hour - Measure of time. Postfix: `h`
    Hour,
    /// Day - Measure of time. Postfix: `d`
    Day,
    /// Year - Measure of time. Postfix: `y`
    Year,
    /// Ampere - Measure of electrical current. Postfix: `A`
    Ampere,
    /// Volt - Measure of electrical voltage. Postfix: `V`
    Volt,
    /// Coulomb - Measure of electrical charge. Postfix: `C`
    Coulomb,
    /// Ohm - Measure of electrical resistance. Postfix: `W`
    Ohm,
    /// Farad - Measure of electrical capacitance. Postfix: `F`
    Farad,
    /// Henry - Measure of electrical inductance. Postfix: `H`
    Henry,
    /// Siemens - Measure of electrical conductance. Postfix: `S`
    Siemens,
    /// Weber - Measure of magnetic flux. Postfix: `Wb`
    Weber,
    /// Tesla - Measure of magnetic flux density. Postfix: `T`
    Tesla,
    /// Kelvin - Measure of thermodynamic temperature. Postfix: `K`
    Kelvin,
    /// Kelvin - Measure of thermodynamic temperature. Postfix: `°C`
    Celsius,
    /// Kelvin - Measure of thermodynamic temperature. Postfix: `°F`
    Fahrenheit,
    /// Candela - Measure of luminous intensity. Postfix: `cd`
    Candela,
    /// Radians - Measure of plane angle. Postfix: `Rad`
    Radian,
    /// Degress - Measure of plane angle. Postfix: `°`
    Degree,
    /// Hertz - Measure of frequency. Postfix: `Hz`
    Hertz,
    /// Joule - Measure of energy. Postfix: `J`
    Joule,
    /// Newton - Measure of force. Postfix: `N`
    Newton,
    /// Kilopond - Measure of force. Postfix: `kp`
    Kilopond,
    /// Pound force - Measure of force. Postfix: `lbf`
    PoundForce,
    /// Watt - Measure of power. Postfix: `W`
    Watt,
    /// Metric horse power - Measure of power. Postfix: `hk`
    MetricHorsePower,
    /// US/UK Horse power - Measure of power. Postfix: `hp`
    UsHorsePower,
    /// Pascal - Measure of pressure. Postfix: `Pa`
    Pascal,
    /// Bar - Measure of pressure. Postfix: `bar`
    Bar,
    /// Atmosphere - Measure of pressure. Postfix: `atm`
    Atmosphere,
    /// Pound force per square inch - Measure of pressure. Postfix: `psi`
    PSI,
    /// Becqerel - Measure of radioactivity. Postfix: `Bq`
    Becqerel,
    /// Lumen - Measure of light lux. Postfix: `lm`
    Lumen,
    /// Lux - Measure of illuminance. Postfix: `lx`
    Lux,
    /// Liter - Measure of volume. Postfix: `l`
    Liter,
    /// British gallon - Measure of volume. **No Postfix or prefix is used**
    UKGallon,
    /// US liquid gallon - Measure of volume. **No Postfix or prefix is used**
    USGallon,
    /// Cubic inch - Measure of volume. Postfix: `cu in`
    CubicInch,
    /// Meter per second - Measure of speed. Postfix: `m/s`
    MeterPerSecond,
    /// Kilometers per hour - Measure of speed. Postfix: `km/s`
    KilometrePerHour,
    /// Miles per hour - Measure of speed. Postfix: `mph`
    MilePerHour,
    /// Revolutions per second - Measure of angular velocity. Postfix: `rps`
    RevolutionsPerSecond,
    /// Revolutions per minute - Measure of angular velocity. Postfix: `rpm`
    RevolutionsPerMinute,
    /// Count. **No Postfix or prefix is used**
    Counts,
    /// Percent. Postfix: `%`
    Percent,
    /// Milligrams per stroke - Measure of mass per engine stroke. Postfix: `mg/stroke`
    MilligramsPerStroke,
    /// Meters per square second - Measure of acceleration. Postfix: `m/s2`
    MeterPerSquareSecond,
    /// Newton meter - Measure of torsion moment. Postfix: `Nm`
    NewtonMeter,
    /// Liters per minute - Measure of flow. Postfix: `l/min`
    LiterPerMinute,
    /// Watts per square meter - Measure of intensity. Postfix: `W/m2`
    WattPerSquareMeter,
    /// Bar per second - Measure of pressure change. Postfix: `bar/s`
    BarPerSecond,
    /// Radians per second - Measure of angular velocity. Postfix: `rad/s2`
    RadiansPerSecond,
    /// Radians per square second - Measure of angular acceleration. Postfix: `rad/s2`
    RadiansPerSquareSecond,
    /// Kilograms per square meter - Postfix: `kg/m2`
    KilogramsPerSquareMeter,
    /// Exa prefix - Prefix: `E`
    Exa,
    /// Peta prefix - Prefix `P`
    Peta,
    /// Tera prefix - Prefix `T`
    Tera,
    /// Giga prefix - Prefix `G`
    Giga,
    /// Mega prefix - Prefix `M`
    Mega,
    /// Kilo prefix - Prefix `k`
    Kilo,
    /// hecto prefix - Prefix `h`
    Hecto,
    /// Deca prefix - Prefix `da`
    Deca,
    /// Deci prefix - Prefix `d`
    Deci,
    /// Centi prefix - Prefix `c`
    Centi,
    /// Milli prefix - Prefix `m`
    Milli,
    /// micro prefix - Prefix `m`
    Micro,
    /// Nano prefix - Prefix `n`
    Nano,
    /// Pico prefix - Prefix `p`
    Pico,
    /// Femto prefix - Prefix `f`
    Femto,
    /// Atto prefix - Prefix: `a`
    Atto,
    /// Year-Month-Day
    Date1,
    /// Day / Month / Year
    Date2,
    /// Month / Day / Year
    Date3,
    /// Calendar week
    Week,
    /// UTC Hour / Minute / Second
    Time1,
    /// Hour / Minute / Second
    Time2,
    /// Second / Minute / Hour / Day / Month / Year
    DateAndTime1,
    /// Second / Minute / Hour / Day / Month / Year / Local minute offset / Local hour offset
    DateAndTime2,
    /// Second / Minute/ Hour / Day / Month / Year
    DateAndTime3,
    /// Second / Minute / Hour / Day / Year / Local minute offset / Local hour offset
    DateAndTime4,
}

/// Scaling byte high nibble encoding
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ScalingByteHigh {
    /// Unsigned numeric integer
    UnsignedNumeric {
        /// Number of bytes making the integer, usually 1-4
        num_bytes: u8
    },
    /// Signed numeric integer
    SignedNumeric {
        /// Number of bytes making the integer, usually 1-4
        num_bytes: u8
    },
    /// Bit mapping encoding to set statuses, without mask
    BitMappingWithoutMask,
    /// Bit mapping encoding to set statuses, with mask
    BitMappingWithMask,
    /// Binary coded decimal encoding
    BCD,
    /// State encoded variable (Enum)
    StateEncodedVariable,
    /// ASCII Text
    ASCII {
        /// Number of bytes stored as ASCII Text
        num_bytes: u8
    },
    /// ANSI 754 signed floating point
    SignedFloatingPoint,
    /// Multiple values data packet
    Packet,
    /// Conversion formula
    Formula,
    /// Unit of presentation format
    UnitOrFormat,
    /// Input / Output state encoding
    StateAndConnectionType,
    /// Reserved or Vehicle manufacturer specific (Unknown)
    Reserved
}

impl From<u8> for ScalingByteHigh {
    fn from(x: u8) -> Self {
        match x & 0xF0 {
            0x00 => Self::UnsignedNumeric { num_bytes: x & 0x0F },
            0x01 => Self::SignedNumeric { num_bytes: x & 0x0F },
            0x02 => Self::BitMappingWithoutMask,
            0x03 => Self::BitMappingWithMask,
            0x04 => Self::BCD,
            0x05 => Self::StateEncodedVariable,
            0x06 => Self::ASCII { num_bytes: x & 0x0F },
            0x07 => Self::SignedFloatingPoint,
            0x08 => Self::Packet,
            0x09 => Self::Formula,
            0x0A => Self::UnitOrFormat,
            0x0B => Self::StateAndConnectionType,
            _ => Self::Reserved
        }
    }
}

impl ScalingByteExtension {
    /// Returns a short string describing the scaling byte extension
    pub fn get_description() -> String {
        todo!()
    }

    /// Returns the optional postfix of the scaling byte
    pub fn get_postfix(&self) -> Option<&'static str> {
        match self {
            ScalingByteExtension::Meter => Some("m"),
            ScalingByteExtension::Foot => Some("ft"),
            ScalingByteExtension::Inch => Some("in"),
            ScalingByteExtension::Yard => Some("yd"),
            ScalingByteExtension::EngMile => Some("mi"),
            ScalingByteExtension::Gram => Some("g"),
            ScalingByteExtension::MetricTon => Some("t"),
            ScalingByteExtension::Second => Some("s"),
            ScalingByteExtension::Minute => Some("min"),
            ScalingByteExtension::Hour => Some("h"),
            ScalingByteExtension::Day => Some("d"),
            ScalingByteExtension::Year => Some("y"),
            ScalingByteExtension::Ampere => Some("A"),
            ScalingByteExtension::Volt => Some("V"),
            ScalingByteExtension::Coulomb => Some("C"),
            ScalingByteExtension::Ohm => Some("W"),
            ScalingByteExtension::Farad => Some("F"),
            ScalingByteExtension::Henry => Some("H"),
            ScalingByteExtension::Siemens => Some("S"),
            ScalingByteExtension::Weber => Some("Wb"),
            ScalingByteExtension::Tesla => Some("T"),
            ScalingByteExtension::Kelvin => Some("K"),
            ScalingByteExtension::Celsius => Some("°C"),
            ScalingByteExtension::Fahrenheit => Some("°F"),
            ScalingByteExtension::Candela => Some("cd"),
            ScalingByteExtension::Radian => Some("rad"),
            ScalingByteExtension::Degree => Some("°"),
            ScalingByteExtension::Hertz => Some("Hz"),
            ScalingByteExtension::Joule => Some("J"),
            ScalingByteExtension::Newton => Some("N"),
            ScalingByteExtension::Kilopond => Some("kp"),
            ScalingByteExtension::PoundForce => Some("lbf"),
            ScalingByteExtension::Watt => Some("W"),
            ScalingByteExtension::MetricHorsePower => Some("hk"),
            ScalingByteExtension::UsHorsePower => Some("hp"),
            ScalingByteExtension::Pascal => Some("Pa"),
            ScalingByteExtension::Bar => Some("bar"),
            ScalingByteExtension::Atmosphere => Some("atm"),
            ScalingByteExtension::PSI => Some("psi"),
            ScalingByteExtension::Becqerel => Some("Bq"),
            ScalingByteExtension::Lumen => Some("lm"),
            ScalingByteExtension::Lux => Some("lx"),
            ScalingByteExtension::Liter => Some("l"),
            ScalingByteExtension::CubicInch => Some("cu in"),
            ScalingByteExtension::MeterPerSecond => Some("m/s"),
            ScalingByteExtension::KilometrePerHour => Some("km/h"),
            ScalingByteExtension::MilePerHour => Some("mph"),
            ScalingByteExtension::RevolutionsPerSecond => Some("rps"),
            ScalingByteExtension::RevolutionsPerMinute => Some("rpm"),
            ScalingByteExtension::Percent => Some("%"),
            ScalingByteExtension::MilligramsPerStroke => Some("mg/stroke"),
            ScalingByteExtension::MeterPerSquareSecond => Some("m/s2"),
            ScalingByteExtension::NewtonMeter => Some("Nm"),
            ScalingByteExtension::LiterPerMinute => Some("l/min"),
            ScalingByteExtension::WattPerSquareMeter => Some("W/m2"),
            ScalingByteExtension::BarPerSecond => Some("bar/s"),
            ScalingByteExtension::RadiansPerSecond => Some("rad/s"),
            ScalingByteExtension::RadiansPerSquareSecond => Some("rad/s2"),
            ScalingByteExtension::KilogramsPerSquareMeter => Some("kg/m2"),
            _ => None
        }
    }

    /// Returns the optional prefix of the scaling byte
    pub fn get_prefix(&self) -> Option<&'static str> {
        match self {
            ScalingByteExtension::Exa => Some("E"),
            ScalingByteExtension::Peta => Some("P"),
            ScalingByteExtension::Tera => Some("T"),
            ScalingByteExtension::Giga => Some("G"),
            ScalingByteExtension::Mega => Some("M"),
            ScalingByteExtension::Kilo => Some("h"),
            ScalingByteExtension::Hecto => Some("h"),
            ScalingByteExtension::Deca => Some("da"),
            ScalingByteExtension::Deci => Some("d"),
            ScalingByteExtension::Centi => Some("c"),
            ScalingByteExtension::Milli => Some("m"),
            ScalingByteExtension::Micro => Some("m"),
            ScalingByteExtension::Nano => Some("n"),
            ScalingByteExtension::Pico => Some("p"),
            ScalingByteExtension::Femto => Some("f"),
            ScalingByteExtension::Atto => Some("a"),
            _ => None
        }
    }
}

impl From<u8> for ScalingByteExtension {
    fn from(x: u8) -> Self {
        match x {
            0x01 => Self::Meter,
            0x02 => Self::Foot,
            0x03 => Self::Inch,
            0x04 => Self::Yard,
            0x05 => Self::EngMile,
            0x06 => Self::Gram,
            0x07 => Self::MetricTon,
            0x08 => Self::Second,
            0x09 => Self::Minute,
            0x0A => Self::Hour,
            0x0B => Self::Day,
            0x0C => Self::Year,
            0x0D => Self::Ampere,
            0x0E => Self::Volt,
            0x0F => Self::Coulomb,

            0x10 => Self::Ohm,
            0x11 => Self::Farad,
            0x12 => Self::Henry,
            0x13 => Self::Siemens,
            0x14 => Self::Weber,
            0x15 => Self::Tesla,
            0x16 => Self::Kelvin,
            0x17 => Self::Celsius,
            0x18 => Self::Fahrenheit,
            0x19 => Self::Candela,
            0x1A => Self::Radian,
            0x1B => Self::Degree,
            0x1C => Self::Hertz,
            0x1D => Self::Joule,
            0x1E => Self::Newton,
            0x1F => Self::Kilopond,

            0x20 => Self::PoundForce,
            0x21 => Self::Watt,
            0x22 => Self::MetricHorsePower,
            0x23 => Self::UsHorsePower,
            0x24 => Self::Pascal,
            0x25 => Self::Bar,
            0x26 => Self::Atmosphere,
            0x27 => Self::PoundForce,
            0x28 => Self::Becqerel,
            0x29 => Self::Lumen,
            0x2A => Self::Lux,
            0x2B => Self::Liter,
            0x2C => Self::UKGallon,
            0x2D => Self::USGallon,
            0x2E => Self::CubicInch,
            0x2F => Self::MeterPerSecond,

            0x30 => Self::KilometrePerHour,
            0x31 => Self::MilePerHour,
            0x32 => Self::RevolutionsPerSecond,
            0x33 => Self::RevolutionsPerMinute,
            0x34 => Self::Counts,
            0x35 => Self::Percent,
            0x36 => Self::MilligramsPerStroke,
            0x37 => Self::MeterPerSquareSecond,
            0x38 => Self::NewtonMeter,
            0x39 => Self::LiterPerMinute,
            0x3A => Self::WattPerSquareMeter,
            0x3B => Self::BarPerSecond,
            0x3C => Self::RadiansPerSecond,
            0x3D => Self::RadiansPerSquareSecond,
            0x3E => Self::KilogramsPerSquareMeter,
            0x3F => Self::NoUnit, // Reserved

            0x40 => Self::Exa,
            0x41 => Self::Peta,
            0x42 => Self::Tera,
            0x43 => Self::Giga,
            0x44 => Self::Mega,
            0x45 => Self::Kilo,
            0x46 => Self::Hecto,
            0x47 => Self::Deca,
            0x48 => Self::Deci,
            0x49 => Self::Centi,
            0x4A => Self::Milli,
            0x4B => Self::Micro,
            0x4C => Self::Nano,
            0x4D => Self::Pico,
            0x4E => Self::Femto,
            0x4F => Self::Atto,

            0x50 => Self::Date1,
            0x51 => Self::Date2,
            0x52 => Self::Date3,
            0x53 => Self::Week,
            0x54 => Self::Time1,
            0x55 => Self::Time2,
            0x56 => Self::DateAndTime1,
            0x57 => Self::DateAndTime2,
            0x58 => Self::DateAndTime3,
            0x59 => Self::DateAndTime4,

            _ => Self::NoUnit
        }
    }
}


/// Represents Scaling data structure returned from ECU
#[derive(Debug, Clone)]
pub struct ScalingData {
    x: f32,
    c0: f32,
    c1: f32,
    c2: f32,
    mapping_byte: u8,
    byte_ext: Vec<ScalingByteExtension>
}

impl ScalingData {
    /// Creates a new scaling data structure
    pub (crate) fn new(x: i32, c0: i32, c1: i32, c2: i32, mapping_byte: u8, byte_ext: &[ScalingByteExtension]) -> Self {
        Self {
            x: x as f32,
            c0: c0 as f32,
            c1: c1 as f32,
            c2: c2 as f32,
            mapping_byte,
            byte_ext: byte_ext.to_vec()
        }
    }

    /// Returns the list of scaling data presentation of the scaling data.
    /// Note that there can be more than one! (EG: Having a prefix and postifx scaling byte)
    pub fn get_scaling_byte(&self) -> &[ScalingByteExtension] {
        &self.byte_ext
    }

    /// Returns a converted value from raw.
    /// If the conversion forumula falls under VMS (Vehicle manufacture specific), then None is returned.
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
            _ => None // VMS or reserved
        }
    }
}