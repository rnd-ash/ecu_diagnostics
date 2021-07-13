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
    UNKNOWN
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Storage state of the DTC
pub enum DTCStatus {

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
    pub mil_on: bool
}