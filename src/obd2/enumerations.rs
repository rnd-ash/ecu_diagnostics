//! Enumeration data for OBD services 01 and 02

use std::fmt::{Debug, Display, Formatter};
use std::mem::transmute;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
/// OBD enumeration type wrapper
pub enum ObdEnumValue {
    /// Fuel system status
    FuelSystemStatus(FuelSystemStatus),
    /// Commanded secondary air status
    CommandedAirStatus(CommandedSecondaryAirStatus),
    /// OBD standard
    ObdStandard(OBDStandard),
    /// Vehicle fuel type
    FuelType(FuelTypeCoding),
}

impl From<ObdEnumValue> for u32 {
    fn from(x: ObdEnumValue) -> u32 {
        let u: u16 = match x {
            ObdEnumValue::FuelSystemStatus(x) => unsafe { transmute(x) },
            ObdEnumValue::CommandedAirStatus(x) => unsafe { transmute(x) },
            ObdEnumValue::ObdStandard(x) => unsafe { transmute(x) },
            ObdEnumValue::FuelType(x) => unsafe { transmute(x) },
        };
        u as u32
    }
}

impl Display for ObdEnumValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ObdEnumValue::FuelSystemStatus(x) => std::fmt::Display::fmt(&x, f),
            ObdEnumValue::CommandedAirStatus(x) => std::fmt::Display::fmt(&x, f),
            ObdEnumValue::ObdStandard(x) => std::fmt::Display::fmt(&x, f),
            ObdEnumValue::FuelType(x) => std::fmt::Display::fmt(&x, f),
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]

/// Fuel system status enumeration for PID 03
pub enum FuelSystemStatus {
    /// Fuel system off
    Off,
    /// Fuel system is in an open loop due to insufficient engine temperature
    OpenLoopLowTemp,
    /// Closed loop and using oxygen sensor feedback to determine fuel mix
    ClosedLoopO2Feedback,
    /// Open loop due to lack of engine load
    OpenLoopEngineLoad,
    /// Open loop due to system failure
    OpenLoopSystemFailure,
    /// Closed loop, using at least one oxygen sensor but there is a fault in the feedback system
    ClosedLoopWithFault,
    /// Invalid fuel system status value
    Invalid(u8),
}

impl Display for FuelSystemStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            FuelSystemStatus::Off => write!(f, "The motor is off"),
            FuelSystemStatus::OpenLoopLowTemp => write!(f, "Open loop due to insufficient engine temperature"),
            FuelSystemStatus::ClosedLoopO2Feedback => write!(f, "Closed loop, using oxygen sensor feedback to determine fuel mix"),
            FuelSystemStatus::OpenLoopEngineLoad => write!(f, "Open loop due to engine load / fuel cut due to deceleration"),
            FuelSystemStatus::OpenLoopSystemFailure => write!(f, "Open loop due to system failure"),
            FuelSystemStatus::ClosedLoopWithFault => write!(f, "Closed loop, using at least one oxygen sensor but there is a fault in the feedback system"),
            FuelSystemStatus::Invalid(x) => write!(f, "Invalid fuel system status 0x{:02X}", x)
        }
    }
}

impl From<u8> for FuelSystemStatus {
    fn from(x: u8) -> Self {
        match x {
            0x00 => Self::Off,
            0x01 => Self::OpenLoopEngineLoad,
            0x02 => Self::ClosedLoopO2Feedback,
            0x04 => Self::OpenLoopEngineLoad,
            0x08 => Self::OpenLoopSystemFailure,
            0x10 => Self::ClosedLoopWithFault,
            x => Self::Invalid(x),
        }
    }
}

/// Commanded secondary air status for PID 12
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum CommandedSecondaryAirStatus {
    /// Upstream
    Upstream,
    /// Downstream of catalytic converter
    DownstreamOfCat,
    /// From the outside atmosphere or off
    FromOutsideOrOff,
    /// Pump commanded on for diagnostics
    PumpCommandedForDiagnostics,
    /// Invalid commanded secondary air status
    Invalid(u8),
}

impl Display for CommandedSecondaryAirStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            CommandedSecondaryAirStatus::Upstream => write!(f, "Upstream"),
            CommandedSecondaryAirStatus::DownstreamOfCat => {
                write!(f, "Downstream of catalytic converter")
            }
            CommandedSecondaryAirStatus::FromOutsideOrOff => {
                write!(f, "From the outside atmosphere or off")
            }
            CommandedSecondaryAirStatus::PumpCommandedForDiagnostics => {
                write!(f, "Pump commanded on for diagnostics")
            }
            CommandedSecondaryAirStatus::Invalid(x) => {
                write!(f, "Invalid commanded secondary air status 0x{:02X}", x)
            }
        }
    }
}

impl From<u8> for CommandedSecondaryAirStatus {
    fn from(x: u8) -> Self {
        match x {
            0x01 => Self::Upstream,
            0x02 => Self::DownstreamOfCat,
            0x04 => Self::FromOutsideOrOff,
            0x08 => Self::PumpCommandedForDiagnostics,
            x => Self::Invalid(x),
        }
    }
}

/// OBD Standard for PID 1C
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum OBDStandard {
    /// OBD-II as defined by the CARB
    OBD_II_CARB,
    /// OBD as defined by the EPA
    OBD_EPA,
    /// OBD and OBD-II
    OBD_OBD_II,
    /// OBD-I
    OBD_I,
    /// Not OBD Compliant
    NON_COMPLIANT,
    /// Europe OBD
    EOBD,
    /// Europe OBD and OBD-II
    EOBD_OBD_II,
    /// Europe OBD and OBD
    EOBD_OBD,
    /// Europe OBD, OBD and OBD-II
    EOBD_OBD_OBD_II,
    /// Japan OBD
    JOBD,
    /// Japan OBD and OBD-II
    JOBD_OBD_II,
    /// Japan OBD and Europe OBD
    JOBD_EOBD,
    /// Japan OBD, Europe OBD and OBD-II
    JOBD_EOBD_OBD_II,
    /// Engine Manufacturer Diagnostics
    EMD,
    /// Engine Manufacturer Diagnostics Enhanced
    EMD_PLUS,
    /// Heavy Duty OBD (Child/Partial)
    HD_OBD_C,
    /// Heavy duty OBD
    HD_OBD,
    /// World wide harmonized OBD
    WWH_OBD,
    /// Heavy duty OBD Stage I without NOx control
    HD_EOBD_I,
    /// Heavy duty OBD Stage I with NOx control
    HD_EOBD_I_N,
    /// Heavy duty OBD Stage II without NOx control
    HD_EOBD_II,
    /// Heavy duty OBD Stage II with NOx control
    HD_EOBD_II_N,
    /// Brazil OBD Phase 1
    OBDBR_1,
    /// Brazil OBD Phase 2
    OBDBR_2,
    /// Korean OBD
    KOBD,
    /// Indian OBD-I
    IOBD_I,
    /// Indian OBD-II
    IOBD_II,
    /// Heavy duty Euro OBD Stage VI
    HD_EOBD_IV,
    /// Reserved for future definition
    Reserved(u8),
    /// Not avaliable for assignment (Illegal argument)
    NotAvailable(u8),
}

impl Display for OBDStandard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            OBDStandard::OBD_II_CARB => write!(f, "OBD-II as defined by the CARB"),
            OBDStandard::OBD_EPA => write!(f, "OBD as defined by the EPA"),
            OBDStandard::OBD_OBD_II => write!(f, "OBD and OBD-II"),
            OBDStandard::OBD_I => write!(f, "OBD-I"),
            OBDStandard::NON_COMPLIANT => write!(f, "Not OBD Compliant"),
            OBDStandard::EOBD => write!(f, "Europe OBD"),
            OBDStandard::EOBD_OBD_II => write!(f, "Europe OBD and OBD-II"),
            OBDStandard::EOBD_OBD => write!(f, "Europe OBD and OBD"),
            OBDStandard::EOBD_OBD_OBD_II => write!(f, "Europe OBD, OBD and OBD-II"),
            OBDStandard::JOBD => write!(f, "Japan OBD"),
            OBDStandard::JOBD_OBD_II => write!(f, "Japan OBD and OBD-II"),
            OBDStandard::JOBD_EOBD => write!(f, "Japan OBD and Europe OBD"),
            OBDStandard::JOBD_EOBD_OBD_II => write!(f, "Japan OBD, Europe OBD and OBD-II"),
            OBDStandard::EMD => write!(f, "Engine Manufacturer Diagnostics"),
            OBDStandard::EMD_PLUS => write!(f, "Engine Manufacturer Diagnostics Enhanced"),
            OBDStandard::HD_OBD_C => write!(f, "Heavy Duty OBD (Child/Partial)"),
            OBDStandard::HD_OBD => write!(f, "Heavy Duty OBD"),
            OBDStandard::WWH_OBD => write!(f, "World wide harmonized OBD"),
            OBDStandard::HD_EOBD_I => write!(f, "Heavy duty OBD Stage I without NOx control"),
            OBDStandard::HD_EOBD_I_N => write!(f, "Heavy duty OBD Stage I with NOx control"),
            OBDStandard::HD_EOBD_II => write!(f, "Heavy duty OBD Stage II without NOx control"),
            OBDStandard::HD_EOBD_II_N => write!(f, "Heavy duty OBD Stage II with NOx control"),
            OBDStandard::OBDBR_1 => write!(f, "Brazil OBD Phase 1"),
            OBDStandard::OBDBR_2 => write!(f, "Brazil OBD Phase 2"),
            OBDStandard::KOBD => write!(f, "Korean OBD"),
            OBDStandard::IOBD_I => write!(f, "Indian OBD-I"),
            OBDStandard::IOBD_II => write!(f, "Indian OBD-II"),
            OBDStandard::HD_EOBD_IV => write!(f, "Heavy duty Euro OBD Stage VI"),
            OBDStandard::Reserved(x) => write!(f, "Reserved 0x{:02X}", x),
            OBDStandard::NotAvailable(x) => write!(f, "Not available for assignment 0x{:02X}", x),
        }
    }
}

impl From<u8> for OBDStandard {
    fn from(x: u8) -> Self {
        match x {
            1 => Self::OBD_II_CARB,
            2 => Self::OBD_EPA,
            3 => Self::OBD_OBD_II,
            4 => Self::OBD_I,
            5 => Self::NON_COMPLIANT,
            6 => Self::EOBD,
            7 => Self::EOBD_OBD_II,
            8 => Self::EOBD_OBD,
            9 => Self::EOBD_OBD_OBD_II,
            10 => Self::JOBD,
            11 => Self::JOBD_OBD_II,
            12 => Self::JOBD_EOBD,
            13 => Self::JOBD_EOBD_OBD_II,
            17 => Self::EMD,
            18 => Self::EMD_PLUS,
            19 => Self::HD_OBD_C,
            20 => Self::HD_OBD,
            21 => Self::WWH_OBD,
            23 => Self::HD_EOBD_I,
            24 => Self::HD_EOBD_I_N,
            25 => Self::HD_EOBD_II,
            26 => Self::HD_EOBD_II_N,
            28 => Self::OBDBR_1,
            29 => Self::OBDBR_2,
            30 => Self::KOBD,
            31 => Self::IOBD_I,
            32 => Self::IOBD_II,
            33 => Self::HD_EOBD_IV,
            14 | 15 | 16 | 22 | 27 | 34..=250 => Self::Reserved(x),
            x => Self::NotAvailable(x),
        }
    }
}

/// Fuel type coding for PID 51
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum FuelTypeCoding {
    /// Fuel type unavailable
    NotAvailable,
    /// Gasoline engine
    Gasoline,
    /// Methanol engine
    Methanol,
    /// Ethanol engine
    Ethanol,
    /// Diesel engine
    Diesel,
    /// LPG engine
    LPG,
    /// CNG engine
    CNG,
    /// Propane engine
    Propane,
    /// Electric engine
    Electric,
    /// Bifuel engine running gasoline as primary
    BifuelGasoline,
    /// Bifuel engine running methanol as primary
    BifuelMethanol,
    /// Bifuel engine running ethanol as primary
    BifuelEthanol,
    /// Bifuel engine running LPG as primary
    BifuelLPG,
    /// Bifuel engine running CNG as primary
    BifuelCNG,
    /// Bifuel engine running propane as primary
    BifuelPropane,
    /// Bifuel engine running electricity as primary
    BifuelElectricity,
    /// Bifuel engine running a electric and combustion engine as primary
    BifuelElectricAndCombustion,
    /// Hybrid gasoline engine
    HybridGasoline,
    /// Hybrid ethanol engine
    HybridEthanol,
    /// Hybrid diesel engine
    HybridDiesel,
    /// Hybrid electric engine
    HybridElectric,
    /// Hybrid electric and combustion engine
    HybridElectricAndCombustion,
    /// Hybrid regenerative engine
    HybridRegen,
    /// Bifuel engine running diesel as primary
    BifuelDiesel,
    /// Reserved fuel type (For future definition)
    Reserved(u8),
}

impl Display for FuelTypeCoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FuelTypeCoding::NotAvailable => write!(f, "Not available"),
            FuelTypeCoding::Gasoline => write!(f, "Gasoline"),
            FuelTypeCoding::Methanol => write!(f, "Methanol"),
            FuelTypeCoding::Ethanol => write!(f, "Ethanol"),
            FuelTypeCoding::Diesel => write!(f, "Diesel"),
            FuelTypeCoding::LPG => write!(f, "LPG"),
            FuelTypeCoding::CNG => write!(f, "CNG"),
            FuelTypeCoding::Propane => write!(f, "Propane"),
            FuelTypeCoding::Electric => write!(f, "Electric"),
            FuelTypeCoding::BifuelGasoline => write!(f, "Bifuel running Gasoline"),
            FuelTypeCoding::BifuelMethanol => write!(f, "Bifuel running Methanol"),
            FuelTypeCoding::BifuelEthanol => write!(f, "Bifuel running Ethanol"),
            FuelTypeCoding::BifuelLPG => write!(f, "Bifuel running LPG"),
            FuelTypeCoding::BifuelCNG => write!(f, "Bifuel running CNG"),
            FuelTypeCoding::BifuelPropane => write!(f, "Bifuel running Propane"),
            FuelTypeCoding::BifuelElectricity => write!(f, "Bifuel running Electricity"),
            FuelTypeCoding::BifuelElectricAndCombustion => {
                write!(f, "Bifuel running electric and combustion engine")
            }
            FuelTypeCoding::HybridGasoline => write!(f, "Hybrid Gasoline"),
            FuelTypeCoding::HybridEthanol => write!(f, "Hybrid Ethanol"),
            FuelTypeCoding::HybridDiesel => write!(f, "hybrid Diesel"),
            FuelTypeCoding::HybridElectric => write!(f, "Hybrid Electric"),
            FuelTypeCoding::HybridElectricAndCombustion => {
                write!(f, "Hybrid running electric and combustion engine")
            }
            FuelTypeCoding::HybridRegen => write!(f, "Hybrid Regenerative"),
            FuelTypeCoding::BifuelDiesel => write!(f, "Bifuel running diesel"),
            FuelTypeCoding::Reserved(x) => write!(f, "Reserved by ISO/SAE 0x{:02X?}", x),
        }
    }
}

impl From<u8> for FuelTypeCoding {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::NotAvailable,
            1 => Self::Gasoline,
            2 => Self::Methanol,
            3 => Self::Ethanol,
            4 => Self::Diesel,
            5 => Self::LPG,
            6 => Self::CNG,
            7 => Self::Propane,
            8 => Self::Electric,
            9 => Self::BifuelGasoline,
            10 => Self::BifuelMethanol,
            11 => Self::BifuelEthanol,
            12 => Self::BifuelLPG,
            13 => Self::BifuelCNG,
            14 => Self::BifuelPropane,
            15 => Self::BifuelElectricity,
            16 => Self::BifuelElectricAndCombustion,
            17 => Self::HybridGasoline,
            18 => Self::HybridEthanol,
            19 => Self::HybridDiesel,
            20 => Self::HybridElectric,
            21 => Self::HybridElectricAndCombustion,
            22 => Self::HybridRegen,
            23 => Self::BifuelDiesel,
            _ => Self::Reserved(x),
        }
    }
}
