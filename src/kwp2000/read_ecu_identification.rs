//! Functions relating to ECU Identification

use std::fmt::format;

use crate::{DiagError, DiagServerResult, helpers::{bcd_decode, bcd_decode_slice}, kwp2000::{KWP2000Command, Kwp2000DiagnosticServer}};


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

    /// Returns the unique ECU ID allocated by DCX for the specific ECU
    pub fn get_dcx_ecu_id(&self) -> u8 {
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
        return (self.0[0] as u16) << 8 | self.0[1] as u16
    }
}

/// Identification structure read with [read_dcs_identification]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DcsEcuIdent {
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
    pub ecu_production_day: u8   
}

/// Identification structure read with [read_dcx_mmc_identification]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DcxMmcEcuIdent {
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
    pub part_number: String
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
    pub tester_serial_number: String
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Identification of a software version on the ECU
pub struct SoftwareBlockIdentification {
    /// ECU origin
    pub origin: u8,
    /// Identification of each block of the software
    pub blocks: Vec<BlockIdentification>
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
    pub part_number: String
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Development data of the ECU. Used by [read_ecu_development_data]
pub struct DevelopmentData {
    /// ECU Processor type
    pub processor_type: u16,
    /// DBkom version. Formatted as XX.YY
    pub communication_matrix: String,
    /// CAN Driver version. Formatted as XX.YY
    pub can_driver_version: String,
    /// NM version. Formatted as XX.YY
    pub nm_version: String,
    /// KWP2000 Module version. Formatted as XX.YY
    pub kwp2000_version: String,
    /// Transport layer version. Formatted as XX.YY
    pub transport_layer_version: String,
    /// DBkom version. Formatted as XX.YY
    pub dbkom_version: String,
    /// Flexer version. Formatted as XX.YY.
    /// 
    /// NOTE: DCA ECUs won't support this, so it'll always be 99.99
    pub flexer_version: String
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// ECU DBCom data. Used by [read_ecu_dbcom_data]
pub struct DBComData {
    /// Memory address (Flash)
    pub memory_address_flash: u32,
    /// Flash data format identifier
    pub data_format_flash: u8,
    /// Uncompressed memory size of Flash
    pub uncompressed_memory_size_flash: u32,
    /// Memory address (RAM)
    pub memory_address_ram: u32,
    /// Uncompressed memory size of RAM
    pub uncompressed_memory_size_ram: u32,
    /// Memory address (EEPROM)
    pub memory_address_eeprom: u32,
    /// Uncompressed memory size of EEPROM
    pub uncompressed_memory_size_eeprom: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Vehicle identification. Used by [read_ecu_vehicle_info]
pub struct VehicleInfo {
    /// Vehicle model year
    pub model_year: u8,
    /// Vehicle code
    pub vehicle_code: u8,
    /// Vehicle body style code
    pub body_style: u8,
    /// Vehicle country code
    pub country_code: u8
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// SDCOM information. Used by [read_system_diag_general_param_data]
pub struct DiagGeneralParamData {
    /// Indicates if global process data exists
    pub global_process_data_exists: bool,
    /// Internal SDCOM communication mode
    pub internal_communication_mode: u8,
    /// SDCOM-SW module version. Formatted as XX.YY
    pub sdcom_version: String,
    /// Year of SDCOM build date
    pub sdcom_build_date_year: u16,
    /// Month of SDCOM build date
    pub sdcom_build_date_month: u8,
    /// Day of SDCOM build date
    pub sdcom_build_date_day: u8,
    /// Version of SDCOM configuration database
    pub sdcom_config_nr: u16,
    /// Version reference number of SDCOM configuration database
    pub sdcom_reference_nr: u16,
    /// Checksum
    pub checksum: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Global diagnostic parameter data. Used by [read_system_diag_global_param_data]
pub struct DiagGlobalParamData {
    /// Number of analog modules active on the ECU
    pub number_of_global_analog_values: u8,
    /// Number of global states
    pub number_of_global_states: u8,
    /// First position within diagnostic CAN Frame
    pub position_in_can_data_frame: u16,
    /// List of process data
    pub process_data: Vec<GlobalProcessData>

}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Global process data
pub struct GlobalProcessData {
    /// Unique data ID for the global process data
    pub data_id: u16,
    /// Timebase of global process data
    pub timebase: u16,
    /// Size of global process data
    pub size: u8
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Diagnostic protocol information. Used by [read_diag_protocol_info]
pub struct DiagProtocolInfo {
    /// Implemented version of KWP2000 specification
    pub kwp2000_requirement_definition: u8,
    /// Implemented version of the flash reprogramming specification
    pub flash_reprogramming_definition_version: u8,
    /// KWP2000 maximum diagnostic level supported
    pub diagnostic_level_support: u8
}

/// Reads DCS ECU identification from ECU
pub fn read_dcs_identification(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DcsEcuIdent> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x86])?;
    if res.len() != 18 {
        return Err(DiagError::InvalidResponseLength)
    }
    Ok(DcsEcuIdent {
        part_number: bcd_decode_slice(&res[2..7], None),
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


/// Reads DCX/MMC ECU identification from ECU
pub fn read_dcx_mmc_identification(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DcxMmcEcuIdent> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x87])?;
    if res.len() != 22 {
        return Err(DiagError::InvalidResponseLength)
    }
    Ok(DcxMmcEcuIdent {
        ecu_origin: res[2],
        supplier: res[3],
        diag_info: DiagnosticInfo([res[4], res[5]]),
        hw_version: bcd_decode_slice(&res[7..=8], Some(".")),
        sw_version: bcd_decode_slice(&res[9..=11], Some(".")),
        part_number: String::from_utf8_lossy(&res[12..]).to_string(),
    })
}

/// Reads the original VIN programmed onto the ECU from the manufacturer
pub fn read_original_vin(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<String> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x88])?;
    Ok(String::from_utf8_lossy(&res[2..]).to_string())
}

/// Reads the unique diagnostic variant code of the ECU
pub fn read_diagnostic_variant_code(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<u32> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x89])?;
    if res.len() != 6 {
        return Err(DiagError::InvalidResponseLength)
    }
    Ok((res[2] as u32) << 24 | (res[3] as u32) << 16 | (res[4] as u32) << 8 | res[5] as u32)
}

/// Reads the current VIN stored on the ECU
pub fn read_current_vin(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<String> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x90])?;
    Ok(String::from_utf8_lossy(&res[2..]).to_string())
}

/// Reads the OBD Calibration ID from the ECU.
pub fn read_calibration_id(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<String> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x96])?;
    Ok(String::from_utf8_lossy(&res[2..]).to_string())
}

/// Reads the calibration verification number from the ECU
pub fn read_cvn(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<[u8; 4]> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x97])?;
    if res.len() != 6 {
        return Err(DiagError::InvalidResponseLength)
    }
    Ok([res[2], res[3], res[4], res[5]])
}


fn decode_module_info(res: &mut Vec<u8>) -> DiagServerResult<ModuleInformation> {
    let active_logical_blocks = res[3];
    res.drain(0..4);
    let mut list_of_blocks: Vec<ModuleBlockInformation> = Vec::new();

    if res.len() % 8 != 0 {
        return Err(DiagError::InvalidResponseLength)
    }

    for i in (0..res.len()).step_by(8) {
        list_of_blocks.push(ModuleBlockInformation {
            tool_supplier_id: res[i],
            programming_date_year: u8::from_str_radix(&bcd_decode(res[i+1]), 10).unwrap(),
            programming_date_month: u8::from_str_radix(&bcd_decode(res[i+2]), 10).unwrap(),
            programming_date_day: u8::from_str_radix(&bcd_decode(res[i+3]), 10).unwrap(),
            tester_serial_number: format!("{:02X}{:02X}{:02X}{:02X}", res[i+4],res[i+5],res[i+6],res[i+7]),
        })
    }

    Ok(ModuleInformation {
        active_logical_blocks,
        module_info: list_of_blocks,
    })
}

/// Reads module information from the ECU's code block
pub fn read_ecu_code_fingerprint(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<ModuleInformation> {
    let mut res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x9A])?;
    decode_module_info(&mut res)
}

/// Reads module information from the ECU's data block
pub fn read_ecu_data_fingerprint(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<ModuleInformation> {
    let mut res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x9B])?;
    decode_module_info(&mut res)
}

fn decode_block_ident(res: &mut Vec<u8>) -> DiagServerResult<SoftwareBlockIdentification> {
    let origin = res[3];

    res.drain(0..4);

    if res.len() % 17 != 0 {
        return Err(DiagError::InvalidResponseLength)
    }

    let mut blocks : Vec<BlockIdentification> = Vec::new();
    for x in (0..res.len()).step_by(17) {
        blocks.push(BlockIdentification {
            supplier_id: res[x],
            diag_info: DiagnosticInfo([res[x+1], res[x+2]]),
            sw_version: bcd_decode_slice(&res[x+4..x+7], Some(".")),
            part_number: bcd_decode_slice(&res[x+8..x+17], None),
        })
    }

    Ok(SoftwareBlockIdentification {
        origin,
        blocks,
    })
}

/// Reads code identification information from the ECU's code block
pub fn read_ecu_code_software_id(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<SoftwareBlockIdentification> {
    let mut res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x9C])?;
    decode_block_ident(&mut res)
}

/// Reads code identification information from the ECU's data block
pub fn read_ecu_data_software_id(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<SoftwareBlockIdentification> {
    let mut res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x9D])?;
    decode_block_ident(&mut res)
}

/// Reads code identification information from the ECU's boot block
pub fn read_ecu_boot_software_id(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<SoftwareBlockIdentification> {
    let mut res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x9E])?;
    decode_block_ident(&mut res)
}
 
/// Reads code identification information from the ECU's boot block
pub fn read_ecu_boot_fingerprint(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<ModuleInformation> {
    let mut res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0x9F])?;
    decode_module_info(&mut res)
}

/// Reads development data from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_development_data(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DevelopmentData> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE0])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads the ECU Serial number. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_serial_number(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<String> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE1])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads DBCom data from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_dbcom_data(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DBComData> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE2])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads the Operating system version on the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_os_version(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<String> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE3])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads reprogramming fault report. The format is binary. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_reprogramming_fault_report(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE4])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads vehicle information from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_vehicle_info(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<VehicleInfo> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE5])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads flash data from block 1. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_flash_info_1(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE6])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads flash data from block 2. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_flash_info_2(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE7])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads general diagnostic parameter data from the ECU (SDCOM). NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_system_diag_general_param_data(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DiagGeneralParamData> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE8])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads global diagnostic parameter data from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_system_diag_global_param_data(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DiagGlobalParamData> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE9])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads the ECU's current configuration status. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_configuration(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xEA])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads ECU protocol information. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_diag_protocol_info(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DiagProtocolInfo> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xEB])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}