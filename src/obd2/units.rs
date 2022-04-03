//! Module for service 01 and 02 unit value type conversions

use crate::obd2::enumerations::ObdEnumValue;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialOrd, PartialEq)]
/// Wrapper type for Service 01 and 02 results
pub struct ObdValue {
    /// Name of the measurement
    name: String,
    /// Value of the measurement
    value: ObdUnitType,
}

impl ObdValue {
    /// Creates a new measurement
    pub fn new<T: Into<String>>(x: T, value: ObdUnitType) -> Self {
        Self {
            name: x.into(),
            value,
        }
    }

    /// Returns the value as a formatted string
    pub fn get_value_as_string(&self, use_metric: bool) -> String {
        match use_metric {
            true => self.value.to_metric_string(),
            false => self.value.to_imperial_string(),
        }
    }

    /// Returns the data in imperial form
    pub fn get_imperial_data(&self) -> f32 {
        self.value.as_imperial()
    }

    /// Returns the data in metric form
    pub fn get_metric_data(&self) -> f32 {
        self.value.as_metric()
    }

    /// Returns the imperial representation unit
    pub fn get_imperial_unit(&self) -> Option<&'static str> {
        self.value.get_imperial_unit()
    }

    /// Returns the metric representation unit
    pub fn get_metric_unit(&self) -> Option<&'static str> {
        self.value.get_metric_unit()
    }

    /// Returns the name of the ObdValue
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Returns inner value
    pub fn get_value(&self) -> ObdUnitType {
        self.value
    }
}

impl Display for ObdValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

/// Wrapper for OBD2 speed values
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Speed(f32); // self.0 is in m/s

impl Speed {
    /// From kilometers per hour
    pub fn from_kmh(kmh: f32) -> Self {
        Self(kmh / 3.6)
    }

    /// From miles per hour
    pub fn from_mph(mph: f32) -> Self {
        Self(mph / 2.237)
    }

    /// Returns the speed in kilometers per hour
    pub fn to_kmh(&self) -> f32 {
        self.0 * 3.6
    }

    /// Returns the speed in miles per hour
    pub fn to_mph(&self) -> f32 {
        self.0 * 2.237
    }

    /// Returns the speed in meters per second
    pub fn to_m_s(&self) -> f32 {
        self.0
    }
}

/// Wrapper for OBD2 temperature values
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Temperature(f32); // self.0 is in *C

impl Temperature {
    /// From celsius
    pub fn from_celsius(c: f32) -> Self {
        Self(c)
    }

    /// From fahrenheit
    pub fn from_fahrenheit(f: f32) -> Self {
        Self((f - 32.0) * (5.0 / 9.0))
    }

    /// Returns the speed in kilometers per hour
    pub fn to_celsius(&self) -> f32 {
        self.0
    }

    /// Returns the speed in miles per hour
    pub fn to_fahrenheit(&self) -> f32 {
        (self.0 * (9.0 / 5.0)) + 32.0
    }
}

/// Wrapper for OBD2 pressure values
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Pressure(f32); // self.0 is in kPa (Kilopascal)

impl Pressure {
    /// From bar
    pub fn from_bar(b: f32) -> Self {
        Self(b * 100.0)
    }

    /// From kPa
    pub fn from_kilo_pascal(kpa: f32) -> Self {
        Self(kpa)
    }

    /// From PSI
    pub fn from_psi(psi: f32) -> Self {
        Self(psi * 68.95)
    }

    /// From Atmosphere
    pub fn from_atmosphere(atmos: f32) -> Self {
        Self(atmos * 1013.25)
    }

    /// To bar
    pub fn to_bar(&self) -> f32 {
        self.0 * 0.01
    }

    /// To kPa
    pub fn to_kilo_pascal(&self) -> f32 {
        self.0
    }

    /// to PSI
    pub fn to_psi(&self) -> f32 {
        self.0 * 0.145038
    }

    /// to atmosphere
    pub fn to_atmosphere(&self) -> f32 {
        self.0 * 0.00986923
    }
}

/// Wrapper for OBD2 time values
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Time(f32); // self.0 is in seconds

impl Time {
    /// From seconds
    pub fn from_seconds(seconds: f32) -> Self {
        Self(seconds)
    }

    /// To seconds
    pub fn to_seconds(&self) -> f32 {
        self.0
    }

    /// To duration. Format string is HH:mm:ss
    pub fn to_elapsed_string(&self) -> String {
        format!(
            "{}:{}:{}",
            (self.0 / 3600.0).floor() as u32,
            ((self.0 / 60.0).floor() as u32) % 60,
            self.0 as u32 % 60
        )
    }
}

/// Wrapper for OBD2 distance values
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Distance(f32); // self.0 is in meters

impl Distance {
    /// From kilometers
    pub fn from_kilometers(km: f32) -> Self {
        Self(km * 1000.0)
    }
    /// To meters
    pub fn to_meters(&self) -> f32 {
        self.0
    }

    /// to Kilometers
    pub fn to_kilometers(&self) -> f32 {
        self.0 / 1000.0
    }

    /// to miles
    pub fn to_miles(&self) -> f32 {
        self.0 / 1609.0
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
/// OBD unit type wrapper
pub enum ObdUnitType {
    /// Raw number
    Raw(f32),
    /// Speed value
    Speed(Speed),
    /// Percentage value
    Percent(f32), // Store as -100..0..100
    /// Temperature value
    Temperature(Temperature),
    /// RPM value
    Rpm(u32),
    /// Volts value
    Volts(f32),
    /// Time value
    Time(Time),
    /// Distance value
    Distance(Distance),
    /// Pressure value
    Pressure(Pressure),
    /// Encoded enumeration value
    Encoded(ObdEnumValue),
}

impl ObdUnitType {
    /// Returns an output string with formatted value in metric form.
    ///
    /// Values are displayed as follows:
    /// * Raw - To 1 decimal place
    /// * RPM - As is
    /// * Speed - In km/h
    /// * Percent - As percentage with 1 decimal place
    /// * Temperature - As degrees celsius
    /// * Volts - As is
    /// * Time - As HH:mm:ss
    /// * Distance - As kilometers
    /// * Pressure - As bar
    /// * Encoded - As is
    pub fn to_metric_string(&self) -> String {
        match self {
            ObdUnitType::Raw(i) => format!("{:.1}", i),
            ObdUnitType::Rpm(i) => format!("{} Rpm", i),
            ObdUnitType::Speed(s) => format!("{} km/h", s.to_kmh()),
            ObdUnitType::Percent(p) => format!("{:.1} %", p),
            ObdUnitType::Temperature(t) => format!("{}°C", t.to_celsius()),
            ObdUnitType::Volts(v) => format!("{}V", v),
            ObdUnitType::Time(t) => t.to_elapsed_string(),
            ObdUnitType::Distance(d) => format!("{} km", d.to_kilometers()),
            ObdUnitType::Pressure(p) => format!("{} bar", p.to_bar()),
            ObdUnitType::Encoded(e) => e.to_string(),
        }
    }

    /// Returns an output string with formatted value in imperial form.
    ///
    /// Values are displayed as follows:
    /// * Raw - To 1 decimal place
    /// * RPM - As is
    /// * Speed - In miles per hour
    /// * Percent - As percentage with 1 decimal place
    /// * Temperature - As degrees fahrenheit
    /// * Volts - As is
    /// * Time - As HH:mm:ss
    /// * Distance - As miles
    /// * Pressure - As PSI
    /// * Encoded - As is
    pub fn to_imperial_string(&self) -> String {
        match self {
            ObdUnitType::Raw(i) => format!("{:.1}", i),
            ObdUnitType::Rpm(i) => format!("{} Rpm", i),
            ObdUnitType::Speed(s) => format!("{} mph", s.to_mph()),
            ObdUnitType::Percent(p) => format!("{:.1} %", p),
            ObdUnitType::Temperature(t) => format!("{} F", t.to_fahrenheit()),
            ObdUnitType::Volts(v) => format!("{}V", v),
            ObdUnitType::Time(t) => t.to_elapsed_string(),
            ObdUnitType::Distance(d) => format!("{} miles", d.to_miles()),
            ObdUnitType::Pressure(p) => format!("{} bar", p.to_psi()),
            ObdUnitType::Encoded(e) => e.to_string(),
        }
    }

    /// Returns the string of the units for the encoded value (If present) for imperial measurement
    ///
    /// Units are as follows (If not specified, there is no unit attached)
    /// Speed - mph
    /// Percent - %
    /// RPM - Rpm
    /// Temperature - *F
    /// Volts - V
    /// Distance - miles
    /// Pressure - psi
    pub fn get_imperial_unit(&self) -> Option<&'static str> {
        match self {
            ObdUnitType::Speed(_) => Some("mph"),
            ObdUnitType::Percent(_) => Some("%"),
            ObdUnitType::Rpm(_) => Some("Rpm"),
            ObdUnitType::Temperature(_) => Some("F"),
            ObdUnitType::Volts(_) => Some("V"),
            ObdUnitType::Distance(_) => Some("miles"),
            ObdUnitType::Pressure(_) => Some("psi"),
            _ => None,
        }
    }

    /// Returns the string of the units for the encoded value (If present) for metric measurement
    ///
    /// Units are as follows (If not specified, there is no unit attached)
    /// Speed - km/h
    /// Percent - %
    /// RPM - Rpm
    /// Temperature - *C
    /// Volts - V
    /// Distance - km
    /// Pressure - bar
    pub fn get_metric_unit(&self) -> Option<&'static str> {
        match self {
            ObdUnitType::Speed(_) => Some("km/h"),
            ObdUnitType::Percent(_) => Some("%"),
            ObdUnitType::Rpm(_) => Some("Rpm"),
            ObdUnitType::Temperature(_) => Some("°C"),
            ObdUnitType::Volts(_) => Some("V"),
            ObdUnitType::Distance(_) => Some("km"),
            ObdUnitType::Pressure(_) => Some("bar"),
            _ => None,
        }
    }

    /// Returns the raw value as a float in imperial form.
    ///
    /// NOTE: encoded enum values are returned as their integer representation!
    pub fn as_imperial(&self) -> f32 {
        match self {
            ObdUnitType::Raw(x) => *x,
            ObdUnitType::Speed(x) => x.to_mph(),
            ObdUnitType::Percent(x) => *x,
            ObdUnitType::Temperature(x) => x.to_fahrenheit(),
            ObdUnitType::Rpm(x) => *x as f32,
            ObdUnitType::Volts(x) => *x,
            ObdUnitType::Time(x) => x.to_seconds(),
            ObdUnitType::Distance(x) => x.to_miles(),
            ObdUnitType::Pressure(x) => x.to_bar(),
            ObdUnitType::Encoded(x) => u32::from(*x) as f32,
        }
    }

    /// Returns the raw value as a float in metric form.
    ///
    /// NOTE: encoded enum values are returned as their integer representation!
    pub fn as_metric(&self) -> f32 {
        match self {
            ObdUnitType::Raw(x) => *x,
            ObdUnitType::Speed(x) => x.to_kmh(),
            ObdUnitType::Percent(x) => *x,
            ObdUnitType::Temperature(x) => x.to_celsius(),
            ObdUnitType::Rpm(x) => *x as f32,
            ObdUnitType::Volts(x) => *x,
            ObdUnitType::Time(x) => x.to_seconds(),
            ObdUnitType::Distance(x) => x.to_kilometers(),
            ObdUnitType::Pressure(x) => x.to_bar(),
            ObdUnitType::Encoded(x) => u32::from(*x) as f32,
        }
    }
}

impl Display for ObdUnitType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_metric_string())
    }
}
