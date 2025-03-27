//! Module for common Diagnostic trouble code data
use bitflags::bitflags;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// DTC name interpretation format specifier
pub enum DTCFormatType {
    /// ISO15031-6 DTC Format
    Iso15031_6,
    /// ISO14229-1 DTC Format
    Iso14229_1,
    /// SAE J1939-73 DTC Format
    SaeJ1939_73,
    /// ISO11992-4 DTC Format
    Iso11992_4,
    /// Unknown DTC Format
    Unknown(u8),
    /// 2 byte hex (KWP2000)
    TwoByteHexKwp,
}

pub(crate) fn dtc_format_from_uds(fmt: u8) -> DTCFormatType {
    match fmt {
        0x00 => DTCFormatType::Iso15031_6,
        0x01 => DTCFormatType::Iso14229_1,
        0x02 => DTCFormatType::SaeJ1939_73,
        0x03 => DTCFormatType::Iso11992_4,
        x => DTCFormatType::Unknown(x),
    }
}

bitflags! {
    /// DTC Status byte according to D.2 of ISO14229
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct DtcStatusByte: u8 {
        /// Most recent check has failed
        const TEST_FAILED                         = 0b00000001;
        /// Test failed at any time in the most recent operation cycle
        const TEST_FAILED_THIS_OPERATION_CYCLE    = 0b00000010;
        /// DTC is pending
        const PENDING_DTC                         = 0b00000100;
        /// DTC is stored
        const CONFIRMED_DTC                       = 0b00001000;
        /// DTC check has not been completed since cleared
        const TEST_NOT_COMPLETED_SINCE_LAST_CLEAR = 0b00010000;
        /// DTC failed since the last clear
        const TEST_FAILED_SINCE_LAST_CLEAR        = 0b00100000;
        /// DTC check has not been completed this operation cycle
        const TEST_NOT_COMPLETED_THIS_OP_CYCLE    = 0b01000000;
        /// Check engine lamp is requested
        const WARNING_INDICATOR_REQUESTED         = 0b10000000;
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
    pub status: DtcStatusByte,
}

impl DTC {
    /// Returns the error in a string format. EG: raw of 8276 = error P
    pub fn get_name_as_string(&self) -> String {
        match self.format {
            DTCFormatType::Iso15031_6 => {
                // 2 bytes
                let b0 = (self.raw >> 8) as u8;
                let b1 = self.raw as u8;
                let component_prefix = match b0 >> 6 {
                    0 => "P",
                    1 => "C",
                    2 => "B",
                    3 => "U",
                    _ => "N", // Should never happen
                };
                format!(
                    "{}{:01X}{:01X}{:01X}{:01X}",
                    component_prefix,
                    ((b0 & 0x30) >> 4),
                    b0 & 0x0F,
                    b1 >> 4,
                    b1 & 0x0F
                )
            }
            DTCFormatType::TwoByteHexKwp => {
                let component_prefix = match (self.raw as u16 & 0b110000000000000) >> 14 {
                    0b00 => "P",
                    0b01 => "C",
                    0b10 => "B",
                    0b11 => "U",
                    _ => "", // Should never happen
                };
                format!("{}{:04X}", component_prefix, self.raw & 0b11111111111111)
                // 14 bits
            }
            _ => format!("{}", self.raw),
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::DTC;

    #[test]
    pub fn test_dtc_parse_raw() {
        let iso15031_6_dtc = DTC {
            format: super::DTCFormatType::Iso15031_6,
            raw: 8276,
            status: super::DtcStatusByte::empty()
        };
        println!("{:04X}", iso15031_6_dtc.raw);
        println!("{}", iso15031_6_dtc.get_name_as_string());
    }
}
