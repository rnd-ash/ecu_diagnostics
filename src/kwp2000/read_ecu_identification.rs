//! Functions relating to ECU Identification

use crate::{
    bcd_decode, bcd_decode_slice, dynamic_diag::DynamicDiagSession, DiagError, DiagServerResult,
};
use automotive_diag::kwp2000::KwpCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]

/// Wrapper for ECU diagnostic information
pub struct DiagnosticInfo([u8; 2]);

impl DiagnosticInfo {
    /// Returns true if the ECU software is production. If this
    /// function returns false, then the software running on the ECU
    /// is considered to be development only
    pub fn is_production_ecu(&self) -> bool {
        self.0[0] & 0b10000000 == 0
    }

    /// Returns the unique ECU ID allocated by Daimler / MMC for the specific ECU
    pub fn get_daimler_mmc_ecu_id(&self) -> u8 {
        self.0[0] & 0b01111111
    }

    /// Returns true if the diagnostic info of the ECU
    /// implies it is currently in boot mode, not being
    /// able to execute the main ECU program
    pub fn is_boot_sw(&self) -> bool {
        self.0[1] >= 0xE0
    }

    /// Returns the entire diagnostic info ID as a u16
    pub fn get_info_id(&self) -> u16 {
        (self.0[0] as u16) << 8 | self.0[1] as u16
    }
}

/// Identification structure read with [Kwp2000DiagnosticServer::read_daimler_identification]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DaimlerEcuIdent {
    /// 10 digital part number
    pub part_number: String,
    /// Week of the year the ECU hardware was produced
    pub ecu_hw_build_week: u8,
    /// Year the ECU hardware was produced
    pub ecu_hw_build_year: u8,

    /// Week of the year the ECU software was compiled
    pub ecu_sw_build_week: u8,
    /// Year the ECU software was compiled
    pub ecu_sw_build_year: u8,

    /// Unique supplier ID (Who manufactured the ECU)
    pub supplier: u8,

    /// Diagnostic information of the ECU
    pub diag_info: DiagnosticInfo,

    /// Year the ECU was manufactured
    pub ecu_production_year: u8,
    /// Month of the year the ECU was manufactured
    pub ecu_production_month: u8,
    /// Day of the month the ECU was manufactured
    pub ecu_production_day: u8,
}

impl DaimlerEcuIdent {
    /// Formats the ECU productions date as dd/mm/yy
    pub fn get_production_date_pretty(&self) -> String {
        format!(
            "{}/{}/{}",
            bcd_decode(self.ecu_production_day),
            bcd_decode(self.ecu_production_month),
            bcd_decode(self.ecu_production_year)
        )
    }

    /// Formats the ECU software build date as ww/yy
    pub fn get_software_date_pretty(&self) -> String {
        format!(
            "{}/{}",
            bcd_decode(self.ecu_sw_build_week),
            bcd_decode(self.ecu_sw_build_year)
        )
    }

    /// Formats the ECU hardware build date as ww/yy
    pub fn get_hardware_date_pretty(&self) -> String {
        format!(
            "{}/{}",
            bcd_decode(self.ecu_hw_build_week),
            bcd_decode(self.ecu_hw_build_year)
        )
    }
}

/// Identification structure read with [Kwp2000DiagnosticServer::read_daimler_mmc_identification]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DaimlerMmcEcuIdent {
    /// Unknown
    pub ecu_origin: u8,
    /// Unique supplier ID (Who manufactured the ECU)
    pub supplier: u8,
    /// Diagnostic information of the ECU
    pub diag_info: DiagnosticInfo,
    /// Hardware version. Formatted as XX_YY
    pub hw_version: String,
    /// Software version. Formatted as XX.YY.ZZ
    pub sw_version: String,
    /// ECU Part number (10 character alpha-numeric string)
    pub part_number: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// ECU Code block fingerprint
pub struct ModuleInformation {
    /// Number logical blocks marked to be erased.
    ///
    /// ## Special values
    /// * 0x00 - Perform no erase
    /// * 0xFE - Erase all blocks
    pub active_logical_blocks: u8,
    /// Information on each block
    pub module_info: Vec<ModuleBlockInformation>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Information on an individual code block on an ECU
pub struct ModuleBlockInformation {
    /// Tool supplier identification who programmed the block
    pub tool_supplier_id: u8,
    /// Programmed year
    pub programming_date_year: u8,
    /// Programmed Month
    pub programming_date_month: u8,
    /// Programmed Day
    pub programming_date_day: u8,
    /// Tester serial number who programmed the block
    pub tester_serial_number: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Identification of a software version on the ECU
pub struct SoftwareBlockIdentification {
    /// ECU origin
    pub origin: u8,
    /// Identification of each block of the software
    pub blocks: Vec<BlockIdentification>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Identification of a software block on the ECU
pub struct BlockIdentification {
    /// ECU Code block supplier info
    pub supplier_id: u8,
    /// Code block Diagnostic information
    pub diag_info: DiagnosticInfo,
    /// Code block Software version. Formatted as XX.YY.ZZ
    pub sw_version: String,
    /// Code block part number
    pub part_number: String,
}

/// Helper function for decoding ECU module info
fn decode_module_info(res: &mut Vec<u8>) -> DiagServerResult<ModuleInformation> {
    let active_logical_blocks = res[3];
    res.drain(0..4);
    let mut list_of_blocks: Vec<ModuleBlockInformation> = Vec::new();

    if res.len() % 8 != 0 {
        return Err(DiagError::InvalidResponseLength);
    }

    for i in (0..res.len()).step_by(8) {
        list_of_blocks.push(ModuleBlockInformation {
            tool_supplier_id: res[i],
            programming_date_year: bcd_decode(res[i + 1]).parse::<u8>().unwrap_or(0),
            programming_date_month: bcd_decode(res[i + 2]).parse::<u8>().unwrap_or(0),
            programming_date_day: bcd_decode(res[i + 3]).parse::<u8>().unwrap_or(0),
            tester_serial_number: format!(
                "{:02X}{:02X}{:02X}{:02X}",
                res[i + 4],
                res[i + 5],
                res[i + 6],
                res[i + 7]
            ),
        })
    }

    Ok(ModuleInformation {
        active_logical_blocks,
        module_info: list_of_blocks,
    })
}

fn decode_block_ident(res: &mut Vec<u8>) -> DiagServerResult<SoftwareBlockIdentification> {
    let origin = res[3];

    res.drain(0..4);

    if res.len() % 17 != 0 {
        return Err(DiagError::InvalidResponseLength);
    }

    let mut blocks: Vec<BlockIdentification> = Vec::new();
    for x in (0..res.len()).step_by(17) {
        blocks.push(BlockIdentification {
            supplier_id: res[x],
            diag_info: DiagnosticInfo([res[x + 1], res[x + 2]]),
            sw_version: bcd_decode_slice(&res[x + 4..x + 7], Some(".")),
            part_number: bcd_decode_slice(&res[x + 8..x + 17], None),
        })
    }

    Ok(SoftwareBlockIdentification { origin, blocks })
}

impl DynamicDiagSession {
    /// Reads Daimler ECU identification from ECU
    pub fn kwp_read_daimler_identification(&self) -> DiagServerResult<DaimlerEcuIdent> {
        let res = self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x86])?;
        if res.len() != 18 {
            return Err(DiagError::InvalidResponseLength);
        }
        Ok(DaimlerEcuIdent {
            part_number: bcd_decode_slice(&res[2..=6], None),
            ecu_hw_build_week: res[7],
            ecu_hw_build_year: res[8],
            ecu_sw_build_week: res[9],
            ecu_sw_build_year: res[10],
            supplier: res[11],
            diag_info: DiagnosticInfo([res[12], res[13]]),
            ecu_production_year: res[15],
            ecu_production_month: res[16],
            ecu_production_day: res[17],
        })
    }

    /// Reads Daimler and MMC ECU identification from ECU
    pub fn kwp_read_daimler_mmc_identification(&self) -> DiagServerResult<DaimlerMmcEcuIdent> {
        let res = self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x87])?;
        if res.len() != 22 {
            return Err(DiagError::InvalidResponseLength);
        }
        Ok(DaimlerMmcEcuIdent {
            ecu_origin: res[2],
            supplier: res[3],
            diag_info: DiagnosticInfo([res[4], res[5]]),
            hw_version: bcd_decode_slice(&res[7..=8], Some(".")),
            sw_version: bcd_decode_slice(&res[9..=11], Some(".")),
            part_number: String::from_utf8_lossy(&res[12..]).to_string(),
        })
    }

    /// Reads the original VIN programmed onto the ECU from the manufacturer
    pub fn kwp_read_original_vin(&self) -> DiagServerResult<String> {
        let res = self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x88])?;
        Ok(String::from_utf8_lossy(&res[2..]).to_string())
    }

    /// Reads the unique diagnostic variant code of the ECU
    pub fn kwp_read_diagnostic_variant_code(&self) -> DiagServerResult<u32> {
        let res = self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x89])?;
        if res.len() != 6 {
            return Err(DiagError::InvalidResponseLength);
        }
        Ok((res[2] as u32) << 24 | (res[3] as u32) << 16 | (res[4] as u32) << 8 | res[5] as u32)
    }

    /// Reads the current VIN stored on the ECU
    pub fn kwp_read_current_vin(&self) -> DiagServerResult<String> {
        let res = self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x90])?;
        Ok(String::from_utf8_lossy(&res[2..]).to_string())
    }

    /// Reads the OBD Calibration ID from the ECU.
    pub fn kwp_read_calibration_id(&self) -> DiagServerResult<String> {
        let res = self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x96])?;
        Ok(String::from_utf8_lossy(&res[2..]).to_string())
    }

    /// Reads the calibration verification number from the ECU
    pub fn kwp_read_cvn(&self) -> DiagServerResult<[u8; 4]> {
        let res = self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x97])?;
        if res.len() != 6 {
            return Err(DiagError::InvalidResponseLength);
        }
        Ok([res[2], res[3], res[4], res[5]])
    }

    /// Reads module information from the ECU's code block
    pub fn kwp_read_ecu_code_fingerprint(&self) -> DiagServerResult<ModuleInformation> {
        let mut res =
            self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x9A])?;
        decode_module_info(&mut res)
    }

    /// Reads module information from the ECU's data block
    pub fn kwp_read_ecu_data_fingerprint(&self) -> DiagServerResult<ModuleInformation> {
        let mut res =
            self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x9B])?;
        decode_module_info(&mut res)
    }

    /// Reads code identification information from the ECU's code block
    pub fn kwp_read_ecu_code_software_id(
        &self,
    ) -> DiagServerResult<SoftwareBlockIdentification> {
        let mut res =
            self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x9C])?;
        decode_block_ident(&mut res)
    }

    /// Reads code identification information from the ECU's data block
    pub fn kwp_read_ecu_data_software_id(
        &self,
    ) -> DiagServerResult<SoftwareBlockIdentification> {
        let mut res =
            self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x9D])?;
        decode_block_ident(&mut res)
    }

    /// Reads code identification information from the ECU's boot block
    pub fn kwp_read_ecu_boot_software_id(
        &self,
    ) -> DiagServerResult<SoftwareBlockIdentification> {
        let mut res =
            self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x9E])?;
        decode_block_ident(&mut res)
    }

    /// Reads code identification information from the ECU's boot block
    pub fn kwp_read_ecu_boot_fingerprint(&self) -> DiagServerResult<ModuleInformation> {
        let mut res =
            self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0x9F])?;
        decode_module_info(&mut res)
    }
}
