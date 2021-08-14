//! Module for common Diagnostic trouble code data

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// DTC name interpretation format specifier
pub enum DTCFormatType {
    /// ISO15031-6 DTC Format
    ISO15031_6,
    /// ISO14229-1 DTC Format
    ISO14229_1,
    /// SAEJ1939-73 DTC Format
    SAEJ1939_73,
    /// ISO11992-4 DTC Format
    ISO11992_4,
    /// Unknown DTC Format
    UNKNOWN(u8),
}

pub(crate) fn dtc_format_from_uds(fmt: u8) -> DTCFormatType {
    match fmt {
        0x00 => DTCFormatType::ISO15031_6,
        0x01 => DTCFormatType::ISO14229_1,
        0x02 => DTCFormatType::SAEJ1939_73,
        0x03 => DTCFormatType::ISO11992_4,
        x => DTCFormatType::UNKNOWN(x),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Storage state of the DTC
pub enum DTCStatus {
    /// No DTC is stored in non volatile memory
    None,
    /// DTC has not met criteria for it to become active or stored,
    /// but a failure condition has been met
    Pending,
    /// DTC is no longer present, but is stored in non volatile memory
    Stored,
    /// DTC is present and stored in non volatile memory
    Active,
    /// Unknown DTC Status
    UNKNOWN(u8),
}

impl DTCStatus {
    pub(crate) fn from_kwp_status(x: u8) -> DTCStatus {
        match (x & 0b01100000) >> 5 {
            0b00 => Self::None,
            0b01 => Self::Stored,
            0b10 => Self::Pending,
            0b11 => Self::Active,
            _ => Self::UNKNOWN(x & 0b01100000), // Should never happen
        }
    }
}

/// Diagnostic trouble code (DTC) storage struct
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DTC {
    /// The [DTCFormatType] of the DTC. This is used
    /// to interpret the raw value of the DTC   
    pub format: DTCFormatType,
    /// The raw value of the DTC according to the ECU
    pub raw: u32,
    /// Status of the DTC
    pub status: DTCStatus,
    /// Indication if the DTC turns on the MIL lamp (Malfunction indicator lamp).
    /// This usually means that the Check engine light is illuminated on the
    /// vehicles instrument cluster
    pub mil_on: bool,
    /// Indication if the DTC conditions have been met since the last clear.
    pub readiness_flag: bool,
}
