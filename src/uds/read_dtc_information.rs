//!  Provides methods to read and query DTCs on the ECU, as well as grabbing Env data about each DTC

use crate::{DiagError, DiagServerResult, dtc::DTC};

use super::{UDSCommand, UdsDiagnosticServer};



#[derive(Debug, Copy, Clone)]
#[repr(u8)]
/// ReadDTCInformation sub-function definitions
pub enum DtcSubFunction {
    /// This function takes a 1 byte DTCStatusMask
    ReportNumberOfDTCByStatusMask = 0x01,
    /// This function takes a 1 byte DTCStatusMask
    ReportDTCByStatusMask = 0x02,
    /// This function takes a 1 byte DTCStatusMask
    ReportMirrorMemoryDTCByStatusMask = 0x0F,
    /// This function takes a 1 byte DTCStatusMask
    ReportNumberOfMirrorMemoryDTCByStatusMask = 0x11,
    /// This function takes a 1 byte DTCStatusMask
    ReportNumberOfEmissionsRelatedOBDDTCByStatusMask = 0x12,
    /// This function takes a 1 byte DTCStatusMask
    ReportEmissionsRelatedOBDDTCByStatusMask = 0x13,

    /// This function takes a 3 byte DTCMaskRecord and a 1 byte DTCSnapshotRecordNumber
    ReportDTCSnapshotIdentifier = 0x03,
    /// This function takes a 3 byte DTCMaskRecord and a 1 byte DTCSnapshotRecordNumber
    ReportDTCSnapshotRecordByDTCNumber = 0x04,

    /// This function takes a 1 byte DTCSnapshotRecordNumber
    ReportDTCSnapshotRecordByRecordNumber = 0x05,

    /// This function take a 3 byte DTCMaskRecord and a 1 byte DTCExtendedDataRecordNumber
    ReportDTCExtendedDataRecordByDTCNumber = 0x06,
    /// This function take a 3 byte DTCMaskRecord and a 1 byte DTCExtendedDataRecordNumber
    ReportMirrorMemoryDTCExtendedDataRecordByDTCNumber = 0x10,

    /// This function takes a 1 byte DTCSeverityMask and a 1 byte DTCStatusMask
    ReportNumberOfDTCBySeverityMaskRecord = 0x07,
    /// This function takes a 1 byte DTCSeverityMask and a 1 byte DTCStatusMask
    ReportDTCBySeverityMaskRecord = 0x08,

    /// This function takes a 3 byte DTCMaskRecord
    ReportSeverityInformationOfDTC = 0x09,

    /// This function take no additional arguments
    ReportSupportedDTC = 0x0A,
    /// This function take no additional arguments
    ReportFirstTestFailedDTC = 0x0B,
    /// This function take no additional arguments
    ReportFirstConfirmedDTC = 0x0C,
    /// This function take no additional arguments
    ReportMostRecentTestFailedDTC = 0x0D,
    /// This function take no additional arguments
    ReportMostRecentConfirmedDTC = 0x0E,
    /// This function take no additional arguments
    ReportDTCFaultDetectionCounter = 0x14,
    /// This function take no additional arguments
    ReportDTCWithPermanentStatus = 0x15
}


/// Returns the number of DTCs stored on the ECU
/// matching the provided status_mask
pub fn get_number_of_dtcs_by_status_mask(server: &mut UdsDiagnosticServer, status_mask: u8) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportNumberOfDTCByStatusMask as u8,
            status_mask
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns a list of DTCs stored on the ECU
/// matching the provided status_mask
pub fn get_dtcs_by_status_mask(server: &mut UdsDiagnosticServer, status_mask: u8) -> DiagServerResult<Vec<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportDTCByStatusMask as u8,
            status_mask
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns a list of DTCs out of the DTC mirror memory whos status_mask matches
/// the provided mask
pub fn get_mirror_memory_dtcs_by_status_mask(server: &mut UdsDiagnosticServer, status_mask: u8) -> DiagServerResult<Vec<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportMirrorMemoryDTCByStatusMask as u8,
            status_mask
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns the number of DTCs in DTC mirror memory who's status_mask matches
/// the provided mask
pub fn get_number_of_mirror_memory_dtcs_by_status_mask(server: &mut UdsDiagnosticServer, status_mask: u8) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportNumberOfMirrorMemoryDTCByStatusMask as u8,
            status_mask
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns the number of OBD emissions related DTCs stored on the ECU
/// who's status mask matches the provided masks
pub fn get_number_of_emissions_related_obd_dtcs_by_status_mask(server: &mut UdsDiagnosticServer, status_mask: u8) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportNumberOfEmissionsRelatedOBDDTCByStatusMask as u8,
            status_mask
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns a list of OBD emissions related DTCs stored on the ECU
/// who's status mask matches the provided mask
pub fn get_emissions_related_obd_dtcs_by_status_mask(server: &mut UdsDiagnosticServer, status_mask: u8) -> DiagServerResult<Vec<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportEmissionsRelatedOBDDTCByStatusMask as u8,
            status_mask
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// 
pub fn get_dtc_snapshot_record_by_dtc_number(server: &mut UdsDiagnosticServer, dtc_mask_record: u32, snapshot_record_number: u8) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportDTCSnapshotRecordByDTCNumber as u8,
            (dtc_mask_record >> 16) as u8,
            (dtc_mask_record >> 8) as u8,
            (dtc_mask_record >> 0) as u8,
            snapshot_record_number
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns all DTC snapshot identifications (DTC number(s) and DTCSnapshot record number(s))
pub fn get_dtc_snapshot_identification(server: &mut UdsDiagnosticServer) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportDTCSnapshotIdentifier as u8,
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns a list of snapshot records based on the mask of snapshot_record_number (0xFF for all records)
pub fn get_dtc_snapshot_record_by_record_number(server: &mut UdsDiagnosticServer, snapshot_record_number: u8) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportDTCSnapshotRecordByRecordNumber as u8,
            snapshot_record_number
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns the DTCExtendedData record(s) asssociated with the provided DTC mask and record number.
/// For the record_number, 0xFE implies all OBD records. and 0xFF implies all records.
/// 
/// ## Returns
/// This function will return the ECUs full response if successful
pub fn get_dtc_extended_data_record_by_dtc_number(server: &mut UdsDiagnosticServer, dtc: u32, extended_data_record_number: u8) -> DiagServerResult<Vec<u8>> {
    server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportDTCExtendedDataRecordByDTCNumber as u8,
            (dtc >> 16) as u8, // High byte
            (dtc >> 8) as u8, // Mid byte
            dtc as u8, // Low byte
            extended_data_record_number
        ]
    )
}


/// Returns a list of extended data records stored in DTC mirror memory for a given DTC.
/// 0xFF for extended_data_record means return all extended data records.
/// 
/// ## Returns
/// This function will return the ECUs full response if successful
pub fn get_mirror_memory_dtc_extended_data_record_by_dtc_number(server: &mut UdsDiagnosticServer, dtc: u32, extended_data_record_number: u8) -> DiagServerResult<Vec<u8>> {
    server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportMirrorMemoryDTCExtendedDataRecordByDTCNumber as u8,
            (dtc >> 16) as u8, // High byte
            (dtc >> 8) as u8, // Mid byte
            dtc as u8, // Low byte
            extended_data_record_number
        ]
    )
}

/// Returns the number of DTCs stored on the ECU that match the provided severity and status mask
pub fn get_number_of_dtcs_by_severity_mask_record(server: &mut UdsDiagnosticServer, severity_mask: u8, status_mask: u8) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportNumberOfDTCBySeverityMaskRecord as u8,
            severity_mask,
            status_mask
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns a list of DTCs who's severity mask matches the provided mask
pub fn get_dtcs_by_severity_mask_record(server: &mut UdsDiagnosticServer, severity_mask: u8, status_mask: u8) -> DiagServerResult<Vec<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportDTCBySeverityMaskRecord as u8,
            severity_mask,
            status_mask
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns the severity status of a provided DTC
pub fn get_severity_information_of_dtc(server: &mut UdsDiagnosticServer, dtc: u32) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation, 
        &[
            DtcSubFunction::ReportSeverityInformationOfDTC as u8,
            (dtc >> 16) as u8,
            (dtc >> 8) as u8,
            (dtc >> 0) as u8,
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns a list of all DTCs that the ECU can return
pub fn get_supported_dtc(server: &mut UdsDiagnosticServer) -> DiagServerResult<Vec<u32>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportSupportedDTC as u8
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns the first failed DTC to be detected since the last DTC clear operation
pub fn get_first_test_failed_dtc(server: &mut UdsDiagnosticServer) -> DiagServerResult<Option<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportFirstTestFailedDTC as u8
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns the first confirmed DTC to be detected since the last DTC clear operation
pub fn get_first_confirmed_dtc(server: &mut UdsDiagnosticServer) -> DiagServerResult<Option<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportFirstConfirmedDTC as u8
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ECU Response was: {:02X?}", resp)))
}

/// Returns the most recent DTC to be detected since the last DTC clear operation
pub fn get_most_recent_test_failed_dtc(server: &mut UdsDiagnosticServer) -> DiagServerResult<Option<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportMostRecentTestFailedDTC as u8
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ReportMostRecentTestFailedDTC ECU Response was: {:02X?}", resp)))
}

/// Returns the most recent DTC to be detected since the last DTC clear operation
pub fn get_most_recent_confirmed_dtc(server: &mut UdsDiagnosticServer) -> DiagServerResult<Option<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportMostRecentConfirmedDTC as u8
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ReportMostRecentConfirmedDTC ECU Response was: {:02X?}", resp)))
}

/// Returns the current number of 'prefailed' DTCs on the ECU, which have not yet been confirmed
/// as being either 'pending' or 'confirmed'
pub fn get_dtc_fault_detection_counter(server: &mut UdsDiagnosticServer) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportDTCFaultDetectionCounter as u8
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ReportDTCFaultDetectionCounter ECU Response was: {:02X?}", resp)))
}

/// Returns a list of DTCs that have a permanent status
pub fn get_dtc_with_permanent_status(server: &mut UdsDiagnosticServer) -> DiagServerResult<Vec<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportDTCWithPermanentStatus as u8
        ]
    )?;
    Err(DiagError::NotImplemented(format!("ReportDTCWithPermanentStatus ECU Response was: {:02X?}", resp)))
}