//!  Provides methods to read and query DTCs on the ECU, as well as grabbing Env data about each DTC

use crate::{
    dtc::{self, DTCFormatType, DTCStatus, DTC},
    DiagError, DiagServerResult,
};

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
    ReportDTCWithPermanentStatus = 0x15,
}

/// Returns the number of DTCs stored on the ECU
/// matching the provided status_mask
///
/// ## Returns
/// Returns a tuple of the given information:
/// 1. (u8) - DTCStatusAvailabilityMask
/// 2. ([DTCFormatType]) - Format of the DTCs
/// 3. (u16) - Number of DTCs which match the status mask
pub fn get_number_of_dtcs_by_status_mask(
    server: &mut UdsDiagnosticServer,
    status_mask: u8,
) -> DiagServerResult<(u8, DTCFormatType, u16)> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportNumberOfDTCByStatusMask as u8,
            status_mask,
        ],
    )?;

    if resp.len() != 6 {
        Err(DiagError::InvalidResponseLength)
    } else {
        server.dtc_format = Some(dtc::dtc_format_from_uds(resp[3]));
        Ok((
            resp[2],
            dtc::dtc_format_from_uds(resp[3]),
            (resp[4] as u16) << 8 | resp[5] as u16,
        ))
    }
}

/// Returns a list of DTCs stored on the ECU
/// matching the provided status_mask
pub fn get_dtcs_by_status_mask(
    server: &mut UdsDiagnosticServer,
    status_mask: u8,
) -> DiagServerResult<Vec<DTC>> {
    let mut resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[DtcSubFunction::ReportDTCByStatusMask as u8, status_mask],
    )?;
    if resp.len() < 7 {
        return Ok(vec![]); // No errors
    }

    resp.drain(0..3);
    if resp.len() % 4 != 0 {
        return Err(DiagError::InvalidResponseLength); // Each DTC should be 4 bytes!
    }

    // Now, see if we can query the ECU's DTC format
    // Note the ECU might not support this command, in which case return 0 as format specifier
    let fmt = match server.dtc_format {
        Some(s) => s,
        None => get_number_of_dtcs_by_status_mask(server, status_mask)
            .map(|r| r.1)
            .unwrap_or(DTCFormatType::UNKNOWN(0)),
    };
    let mut result: Vec<DTC> = Vec::new();

    for x in (0..resp.len()).step_by(4) {
        let dtc_code: u32 = (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
        let status = resp[x + 3];

        result.push(DTC {
            format: fmt,
            raw: dtc_code,
            status: DTCStatus::UNKNOWN(status), // TODO
            mil_on: status & 0b10000000 != 0,
        })
    }

    Ok(result)
}

/// Returns a list of DTCs out of the DTC mirror memory whos status_mask matches
/// the provided mask
pub fn get_mirror_memory_dtcs_by_status_mask(
    server: &mut UdsDiagnosticServer,
    status_mask: u8,
) -> DiagServerResult<Vec<DTC>> {
    let mut resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportMirrorMemoryDTCByStatusMask as u8,
            status_mask,
        ],
    )?;
    if resp.len() < 7 {
        return Ok(vec![]); // No errors
    }

    resp.drain(0..3);
    if resp.len() % 4 != 0 {
        return Err(DiagError::InvalidResponseLength); // Each DTC should be 4 bytes!
    }

    // Now, see if we can query the ECU's DTC format
    // Note the ECU might not support this command, in which case return 0 as format specifier
    let fmt = match server.dtc_format {
        Some(s) => s,
        None => get_number_of_dtcs_by_status_mask(server, status_mask)
            .map(|r| r.1)
            .unwrap_or(DTCFormatType::UNKNOWN(0)),
    };
    let mut result: Vec<DTC> = Vec::new();

    for x in (0..resp.len()).step_by(4) {
        let dtc_code: u32 = (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
        let status = resp[x + 3];

        result.push(DTC {
            format: fmt,
            raw: dtc_code,
            status: DTCStatus::UNKNOWN(status), // TODO
            mil_on: status & 0b10000000 != 0,
        })
    }
    Ok(result)
}

/// Returns the number of DTCs in DTC mirror memory who's status_mask matches
/// the provided mask
///
/// ## Returns
/// Returns a tuple of the given information:
/// 1. (u8) - DTCStatusAvailabilityMask
/// 2. ([DTCFormatType]) - Format of the DTCs
/// 3. (u16) - Number of DTCs which match the status mask
pub fn get_number_of_mirror_memory_dtcs_by_status_mask(
    server: &mut UdsDiagnosticServer,
    status_mask: u8,
) -> DiagServerResult<(u8, DTCFormatType, u16)> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportNumberOfMirrorMemoryDTCByStatusMask as u8,
            status_mask,
        ],
    )?;
    if resp.len() != 6 {
        Err(DiagError::InvalidResponseLength)
    } else {
        server.dtc_format = Some(dtc::dtc_format_from_uds(resp[3]));
        Ok((
            resp[2],
            dtc::dtc_format_from_uds(resp[3]),
            (resp[4] as u16) << 8 | resp[5] as u16,
        ))
    }
}

/// Returns the number of OBD emissions related DTCs stored on the ECU
/// who's status mask matches the provided masks
///
/// ## Returns
/// Returns a tuple of the given information:
/// 1. (u8) - DTCStatusAvailabilityMask
/// 2. ([DTCFormatType]) - Format of the DTCs
/// 3. (u16) - Number of DTCs which match the status mask
pub fn get_number_of_emissions_related_obd_dtcs_by_status_mask(
    server: &mut UdsDiagnosticServer,
    status_mask: u8,
) -> DiagServerResult<(u8, DTCFormatType, u16)> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportNumberOfEmissionsRelatedOBDDTCByStatusMask as u8,
            status_mask,
        ],
    )?;
    if resp.len() != 6 {
        Err(DiagError::InvalidResponseLength)
    } else {
        server.dtc_format = Some(dtc::dtc_format_from_uds(resp[3]));
        Ok((
            resp[2],
            dtc::dtc_format_from_uds(resp[3]),
            (resp[4] as u16) << 8 | resp[5] as u16,
        ))
    }
}

/// Returns a list of OBD emissions related DTCs stored on the ECU
/// who's status mask matches the provided mask
pub fn get_emissions_related_obd_dtcs_by_status_mask(
    server: &mut UdsDiagnosticServer,
    status_mask: u8,
) -> DiagServerResult<Vec<DTC>> {
    let mut resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportEmissionsRelatedOBDDTCByStatusMask as u8,
            status_mask,
        ],
    )?;
    if resp.len() < 7 {
        return Ok(vec![]); // No errors
    }

    resp.drain(0..3);
    if resp.len() % 4 != 0 {
        return Err(DiagError::InvalidResponseLength); // Each DTC should be 4 bytes!
    }

    // Now, see if we can query the ECU's DTC format
    // Note the ECU might not support this command, in which case return 0 as format specifier
    let fmt = match server.dtc_format {
        Some(s) => s,
        None => get_number_of_dtcs_by_status_mask(server, status_mask)
            .map(|r| r.1)
            .unwrap_or(DTCFormatType::UNKNOWN(0)),
    };
    let mut result: Vec<DTC> = Vec::new();

    for x in (0..resp.len()).step_by(4) {
        let dtc_code: u32 = (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
        let status = resp[x + 3];

        result.push(DTC {
            format: fmt,
            raw: dtc_code,
            status: DTCStatus::UNKNOWN(status), // TODO
            mil_on: status & 0b10000000 != 0,
        })
    }
    Ok(result)
}

///
pub fn get_dtc_snapshot_record_by_dtc_number(
    server: &mut UdsDiagnosticServer,
    dtc_mask_record: u32,
    snapshot_record_number: u8,
) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportDTCSnapshotRecordByDTCNumber as u8,
            (dtc_mask_record >> 16) as u8,
            (dtc_mask_record >> 8) as u8,
            dtc_mask_record as u8,
            snapshot_record_number,
        ],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns all DTC snapshot identifications (DTC number(s) and DTCSnapshot record number(s))
pub fn get_dtc_snapshot_identification(server: &mut UdsDiagnosticServer) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[DtcSubFunction::ReportDTCSnapshotIdentifier as u8],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns a list of snapshot records based on the mask of snapshot_record_number (0xFF for all records)
pub fn get_dtc_snapshot_record_by_record_number(
    server: &mut UdsDiagnosticServer,
    snapshot_record_number: u8,
) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportDTCSnapshotRecordByRecordNumber as u8,
            snapshot_record_number,
        ],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns the DTCExtendedData record(s) asssociated with the provided DTC mask and record number.
/// For the record_number, 0xFE implies all OBD records. and 0xFF implies all records.
///
/// ## Returns
/// This function will return the ECUs full response if successful
pub fn get_dtc_extended_data_record_by_dtc_number(
    server: &mut UdsDiagnosticServer,
    dtc: u32,
    extended_data_record_number: u8,
) -> DiagServerResult<Vec<u8>> {
    server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportDTCExtendedDataRecordByDTCNumber as u8,
            (dtc >> 16) as u8, // High byte
            (dtc >> 8) as u8,  // Mid byte
            dtc as u8,         // Low byte
            extended_data_record_number,
        ],
    )
}

/// Returns a list of extended data records stored in DTC mirror memory for a given DTC.
/// 0xFF for extended_data_record means return all extended data records.
///
/// ## Returns
/// This function will return the ECUs full response if successful
pub fn get_mirror_memory_dtc_extended_data_record_by_dtc_number(
    server: &mut UdsDiagnosticServer,
    dtc: u32,
    extended_data_record_number: u8,
) -> DiagServerResult<Vec<u8>> {
    server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportMirrorMemoryDTCExtendedDataRecordByDTCNumber as u8,
            (dtc >> 16) as u8, // High byte
            (dtc >> 8) as u8,  // Mid byte
            dtc as u8,         // Low byte
            extended_data_record_number,
        ],
    )
}

/// Returns the number of DTCs stored on the ECU that match the provided severity and status mask
pub fn get_number_of_dtcs_by_severity_mask_record(
    server: &mut UdsDiagnosticServer,
    severity_mask: u8,
    status_mask: u8,
) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportNumberOfDTCBySeverityMaskRecord as u8,
            severity_mask,
            status_mask,
        ],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns a list of DTCs who's severity mask matches the provided mask
pub fn get_dtcs_by_severity_mask_record(
    server: &mut UdsDiagnosticServer,
    severity_mask: u8,
    status_mask: u8,
) -> DiagServerResult<Vec<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportDTCBySeverityMaskRecord as u8,
            severity_mask,
            status_mask,
        ],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns the severity status of a provided DTC
pub fn get_severity_information_of_dtc(
    server: &mut UdsDiagnosticServer,
    dtc: u32,
) -> DiagServerResult<u32> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[
            DtcSubFunction::ReportSeverityInformationOfDTC as u8,
            (dtc >> 16) as u8,
            (dtc >> 8) as u8,
            dtc as u8,
        ],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns a list of all DTCs that the ECU can return
pub fn get_supported_dtc(server: &mut UdsDiagnosticServer) -> DiagServerResult<Vec<DTC>> {
    let mut resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[DtcSubFunction::ReportSupportedDTC as u8],
    )?;
    if resp.len() < 7 {
        return Ok(vec![]); // No errors
    }

    resp.drain(0..3);
    if resp.len() % 4 != 0 {
        return Err(DiagError::InvalidResponseLength); // Each DTC should be 4 bytes!
    }

    // Now, see if we can query the ECU's DTC format
    // Note the ECU might not support this command, in which case return 0 as format specifier
    let fmt = match server.dtc_format {
        Some(s) => s,
        None => get_number_of_dtcs_by_status_mask(server, 0xFF)
            .map(|r| r.1)
            .unwrap_or(DTCFormatType::UNKNOWN(0)),
    };
    let mut result: Vec<DTC> = Vec::new();

    for x in (0..resp.len()).step_by(4) {
        let dtc_code: u32 = (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
        let status = resp[x + 3];

        result.push(DTC {
            format: fmt,
            raw: dtc_code,
            status: DTCStatus::UNKNOWN(status), // TODO
            mil_on: status & 0b10000000 != 0,
        })
    }
    Ok(result)
}

/// Returns the first failed DTC to be detected since the last DTC clear operation
pub fn get_first_test_failed_dtc(
    server: &mut UdsDiagnosticServer,
) -> DiagServerResult<Option<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[DtcSubFunction::ReportFirstTestFailedDTC as u8],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns the first confirmed DTC to be detected since the last DTC clear operation
pub fn get_first_confirmed_dtc(server: &mut UdsDiagnosticServer) -> DiagServerResult<Option<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[DtcSubFunction::ReportFirstConfirmedDTC as u8],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns the most recent DTC to be detected since the last DTC clear operation
pub fn get_most_recent_test_failed_dtc(
    server: &mut UdsDiagnosticServer,
) -> DiagServerResult<Option<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[DtcSubFunction::ReportMostRecentTestFailedDTC as u8],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ReportMostRecentTestFailedDTC ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns the most recent DTC to be detected since the last DTC clear operation
pub fn get_most_recent_confirmed_dtc(
    server: &mut UdsDiagnosticServer,
) -> DiagServerResult<Option<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[DtcSubFunction::ReportMostRecentConfirmedDTC as u8],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ReportMostRecentConfirmedDTC ECU Response was: {:02X?}",
        resp
    )))
}

/// Returns the current number of 'prefailed' DTCs on the ECU, which have not yet been confirmed
/// as being either 'pending' or 'confirmed'
///
/// ## Returns
/// This function will return a vector of information, where each element is a tuple containing the following values:
/// 1. (u32) - DTC Code
/// 2. (u8) - Fault detection counter
pub fn get_dtc_fault_detection_counter(
    server: &mut UdsDiagnosticServer,
) -> DiagServerResult<Vec<(u32, u8)>> {
    let mut resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[DtcSubFunction::ReportDTCFaultDetectionCounter as u8],
    )?;
    if resp.len() < 6 {
        return Ok(vec![]); // No errors
    }

    resp.drain(0..2);
    if resp.len() % 4 != 0 {
        return Err(DiagError::InvalidResponseLength); // Each DTC should be 4 bytes!
    }

    let mut result: Vec<(u32, u8)> = Vec::new();

    for x in (0..resp.len()).step_by(4) {
        let dtc_code: u32 = (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
        result.push((dtc_code, resp[x + 3]))
    }
    Ok(result)
}

/// Returns a list of DTCs that have a permanent status
pub fn get_dtc_with_permanent_status(
    server: &mut UdsDiagnosticServer,
) -> DiagServerResult<Vec<DTC>> {
    let resp = server.execute_command_with_response(
        UDSCommand::ReadDTCInformation,
        &[DtcSubFunction::ReportDTCWithPermanentStatus as u8],
    )?;
    Err(DiagError::NotImplemented(format!(
        "ReportDTCWithPermanentStatus ECU Response was: {:02X?}",
        resp
    )))
}

#[cfg(test)]
pub mod sim_ecu_test {
    use crate::uds::uds_test::{FakeIsoTpChannel, TestUdsServer};

    use super::*;

    #[test]
    fn get_dtcs_by_status_mask() {
        let mut fake_ecu = FakeIsoTpChannel::new();
        fake_ecu.add_sid_respose(0x19, Some(0x01), &[0x7B, 0x01, 0x00, 0x0C]);
        fake_ecu.add_sid_respose(
            0x19,
            Some(0x02),
            &[
                0x7B, 0x06, 0x10, 0x00, 0x28, 0xA1, 0xDC, 0x01, 0x69, 0xD1, 0x60, 0x00, 0x28, 0x17,
                0x2C, 0x13, 0x40, 0x9A, 0x39, 0x87, 0x50, 0xA1, 0x0A, 0x00, 0x40, 0xA1, 0x0B, 0x00,
                0x40, 0xA2, 0x01, 0x00, 0x40, 0xC1, 0x22, 0x08, 0x40, 0xC1, 0x22, 0x87, 0x40, 0xD1,
                0x98, 0x00, 0x40, 0xD4, 0x0F, 0x00, 0x40,
            ],
        );
        let mut s = TestUdsServer::new(fake_ecu);
        let result = super::get_dtcs_by_status_mask(&mut s.uds, 0xFF);
        println!("{:?}", result);
    }
}
