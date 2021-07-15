//! Read data by Local identifier

use crate::{DiagError, DiagServerResult, kwp2000::KWP2000Command};

use super::Kwp2000DiagnosticServer;

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

/// Reads development data from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_development_data(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DevelopmentData> {
    let res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xE0])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads the ECU Serial number.
/// 
/// This function returns the bytes of just the serial number of the ECU, which 
/// can be interpreted as either ASCII (DCA ECUs), or Model line specification (Varies from OEM to OEM)
pub fn read_ecu_serial_number(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let mut res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xE1])?;
    res.drain(0..2);
    Ok(res)
}

/// Reads DBCom data from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_dbcom_data(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DBComData> {
    let res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xE2])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads the Operating system version on the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_os_version(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<String> {
    let res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xE3])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads reprogramming fault report. The format is binary.
/// This is to be interpreted by GSP/SDE.
pub fn read_ecu_reprogramming_fault_report(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let mut res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xE4])?;
    res.drain(0..2);
    Ok(res)
}

/// Reads vehicle information from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_vehicle_info(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<VehicleInfo> {
    let res = server.execute_command_with_response(KWP2000Command::ReadECUIdentification, &[0xE5])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads flash data from block 1. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_flash_info_1(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xE6])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads flash data from block 2. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_flash_info_2(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xE7])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads general diagnostic parameter data from the ECU (SDCOM). NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_system_diag_general_param_data(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DiagGeneralParamData> {
    let res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xE8])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads global diagnostic parameter data from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_system_diag_global_param_data(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DiagGlobalParamData> {
    let res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xE9])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads the ECU's current configuration status. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_ecu_configuration(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<Vec<u8>> {
    let res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xEA])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads ECU protocol information. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
pub fn read_diag_protocol_info(server: &mut Kwp2000DiagnosticServer) -> DiagServerResult<DiagProtocolInfo> {
    let res = server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[0xEB])?;
    Err(DiagError::NotImplemented(format!("ECU Response: {:02X?}", res)))
}

/// Reads data from a custom local identifier 
/// 
/// ## Supported local identifier ranges
/// * 0x01-0x7F - Record local identifier
/// * 0xA0-0xDF - Record local identifier
/// * 0xF0-0xF9 - Dynamically defined local identifier
/// 
/// ## Important notes:
/// 1. Do NOT use Identifiers between 0x80-0x9F. These are for [crate::kwp2000::read_ecu_identification] only!
/// 2. Identifiers between 0xE0 and 0xEB are handled by the other functions in this module, and return
/// the data as parsed responses
pub fn read_custom_local_identifier(server: &mut Kwp2000DiagnosticServer, local_identifier: u8) -> DiagServerResult<Vec<u8>> {
    server.execute_command_with_response(KWP2000Command::ReadDataByLocalIdentifier, &[local_identifier])
}