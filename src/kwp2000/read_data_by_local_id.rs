//! Read data by Local identifier

use crate::{dynamic_diag::DynamicDiagSession, DiagError, DiagServerResult};
use automotive_diag::kwp2000::KwpCommand;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Development data of the ECU. Used by [super::Kwp2000DiagnosticServer::read_ecu_development_data]
pub struct DevelopmentData {
    /// ECU Processor type
    pub processor_type: u16,
    /// Database communication matrix version. Formatted as XX.YY
    pub db_comm_matrix_version: String,
    /// CAN Driver version. Formatted as XX.YY
    pub can_driver_version: String,
    /// NM version. Formatted as XX.YY
    pub nm_version: String,
    /// KWP2000 Module version. Formatted as XX.YY
    pub kwp2000_version: String,
    /// Transport layer version. Formatted as XX.YY
    pub transport_layer_version: String,
    /// Database communication version. Formatted as XX.YY
    pub db_comm_version: String,
    /// Flexer version. Formatted as XX.YY.
    /// If the ECU does not support this data field, `99.99` will be stored
    pub flexer_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// ECU DBCom data. Used by [super::Kwp2000DiagnosticServer::read_ecu_dbcom_data]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Vehicle identification. Used by [super::Kwp2000DiagnosticServer::read_ecu_vehicle_info]
pub struct VehicleInfo {
    /// Vehicle model year
    pub model_year: u8,
    /// Vehicle code
    pub vehicle_code: u8,
    /// Vehicle body style code
    pub body_style: u8,
    /// Vehicle country code
    pub country_code: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// SDCOM information. Used by [super::Kwp2000DiagnosticServer::read_system_diag_general_param_data]
pub struct DiagGeneralParamData {
    /// Indicates if global process data exists
    pub global_process_data_exists: bool,
    /// Internal communication mode
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
/// Global diagnostic parameter data. Used by [super::Kwp2000DiagnosticServer::read_system_diag_global_param_data]
pub struct DiagGlobalParamData {
    /// Number of analog modules active on the ECU
    pub number_of_global_analog_values: u8,
    /// Number of global states
    pub number_of_global_states: u8,
    /// First position within diagnostic CAN Frame
    pub position_in_can_data_frame: u16,
    /// List of process data
    pub process_data: Vec<GlobalProcessData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Global process data
pub struct GlobalProcessData {
    /// Unique data ID for the global process data
    pub data_id: u16,
    /// Timebase of global process data
    pub timebase: u16,
    /// Size of global process data
    pub size: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Diagnostic protocol information. Used by [Kwp2000DiagnosticServer::read_diag_protocol_info]
pub struct DiagProtocolInfo {
    /// Implemented version of KWP2000 specification
    pub kwp2000_requirement_definition: u8,
    /// Implemented version of the flash reprogramming specification
    pub flash_reprogramming_definition_version: u8,
    /// KWP2000 maximum diagnostic level supported
    pub diagnostic_level_support: u8,
}

impl DynamicDiagSession {
    /// Reads development data from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_ecu_development_data(&self) -> DiagServerResult<DevelopmentData> {
        let res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xE0])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads the ECU Serial number.
    ///
    /// This function returns the bytes of just the serial number of the ECU, which
    /// can be interpreted as either ASCII (Daimler ECUs), or Model line specification (Varies from OEM to OEM)
    pub fn kwp_read_ecu_serial_number(&self) -> DiagServerResult<Vec<u8>> {
        let mut res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xE1])?;
        res.drain(0..2);
        Ok(res)
    }

    /// Reads DBCom data from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_ecu_dbcom_data(&self) -> DiagServerResult<DBComData> {
        let res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xE2])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads the Operating system version on the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_ecu_os_version(&self) -> DiagServerResult<String> {
        let res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xE3])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads reprogramming fault report. The format is binary.
    /// This is to be interpreted by GSP/SDE.
    pub fn kwp_read_ecu_reprogramming_fault_report(&self) -> DiagServerResult<Vec<u8>> {
        let mut res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xE4])?;
        res.drain(0..2);
        Ok(res)
    }

    /// Reads vehicle information from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_ecu_vehicle_info(&self) -> DiagServerResult<VehicleInfo> {
        let res = self.send_command_with_response(KwpCommand::ReadECUIdentification, &[0xE5])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads flash data from block 1. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_ecu_flash_info_1(&self) -> DiagServerResult<Vec<u8>> {
        let res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xE6])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads flash data from block 2. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_ecu_flash_info_2(&self) -> DiagServerResult<Vec<u8>> {
        let res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xE7])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads general diagnostic parameter data from the ECU (SDCOM). NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_system_diag_general_param_data(
        &self,
    ) -> DiagServerResult<DiagGeneralParamData> {
        let res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xE8])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads global diagnostic parameter data from the ECU. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_system_diag_global_param_data(&self) -> DiagServerResult<DiagGlobalParamData> {
        let res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xE9])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads the ECU's current configuration status. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_ecu_configuration(&self) -> DiagServerResult<Vec<u8>> {
        let res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xEA])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads ECU protocol information. NOT IMPLEMENTED YET (Will return [DiagError::NotImplemented])
    pub fn kwp_read_diag_protocol_info(&self) -> DiagServerResult<DiagProtocolInfo> {
        let res =
            self.send_command_with_response(KwpCommand::ReadDataByLocalIdentifier, &[0xEB])?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response: {res:02X?}"
        )))
    }

    /// Reads data from a custom local identifier
    ///
    /// ## Supported local identifier ranges
    /// * 0x01-0x7F - Record local identifier
    /// * 0xA0-0xDF - Record local identifier
    /// * 0xF0-0xF9 - Dynamically defined local identifier
    ///
    /// ## Important notes:
    /// 1. If the ECU supports commands for identification purposes, then asking for an identifier in the range of 0x86-0x9F will
    ///     return ECU ident data.
    pub fn kwp_read_custom_local_identifier(
        &self,
        local_identifier: u8,
    ) -> DiagServerResult<Vec<u8>> {
        let mut res = self.send_command_with_response(
            KwpCommand::ReadDataByLocalIdentifier,
            &[local_identifier],
        )?;
        // Now check identifier in response message was same as our request identifier, if so, strip it
        // from the response message
        if res.len() < 2 {
            // Require Positive SID, IDENT
            return Err(DiagError::InvalidResponseLength);
        }
        if res[1] != local_identifier {
            return Err(DiagError::MismatchedIdentResponse {
                want: local_identifier as _,
                received: res[1] as _,
            });
        }
        res.drain(0..2);
        Ok(res)
    }
}
