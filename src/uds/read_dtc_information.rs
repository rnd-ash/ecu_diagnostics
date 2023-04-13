//!  Provides methods to read and query DTCs on the ECU, as well as grabbing Env data about each DTC

use crate::{
    dtc::{self, DTCFormatType, DTCStatus, DTC},
    DiagError, DiagServerResult, dynamic_diag::DynamicDiagSession,
};

pub use auto_uds::DtcSubFunction;

impl DynamicDiagSession {
    /// Returns the number of DTCs stored on the ECU
    /// matching the provided status_mask
    ///
    /// ## Returns
    /// Returns a tuple of the given information:
    /// 1. (u8) - DTCStatusAvailabilityMask
    /// 2. ([DTCFormatType]) - Format of the DTCs
    /// 3. (u16) - Number of DTCs which match the status mask
    pub fn uds_get_number_of_dtcs_by_status_mask(
        &mut self,
        status_mask: u8,
    ) -> DiagServerResult<(u8, DTCFormatType, u16)> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportNumberOfDtcByStatusMask as u8,
                status_mask,
            ],
        )?;

        if resp.len() != 6 {
            Err(DiagError::InvalidResponseLength)
        } else {
            Ok((
                resp[2],
                dtc::dtc_format_from_uds(resp[3]),
                (resp[4] as u16) << 8 | resp[5] as u16,
            ))
        }
    }

    /// Returns a list of DTCs stored on the ECU
    /// matching the provided status_mask
    pub fn uds_get_dtcs_by_status_mask(&mut self, status_mask: u8) -> DiagServerResult<Vec<DTC>> {
        let mut resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
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
        let fmt = self.uds_get_number_of_dtcs_by_status_mask(status_mask)
            .map(|x| x.1)
            .unwrap_or(DTCFormatType::Unknown(0));
        let mut result: Vec<DTC> = Vec::new();

        for x in (0..resp.len()).step_by(4) {
            let dtc_code: u32 =
                (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
            let status = resp[x + 3];

            result.push(DTC {
                format: fmt,
                raw: dtc_code,
                status: DTCStatus::Unknown(status), // TODO
                mil_on: status & 0b10000000 != 0,
                readiness_flag: false,
            })
        }

        Ok(result)
    }

    /// Returns a list of DTCs out of the DTC mirror memory who's status_mask matches
    /// the provided mask
    pub fn uds_get_mirror_memory_dtcs_by_status_mask(
        &mut self,
        status_mask: u8,
    ) -> DiagServerResult<Vec<DTC>> {
        let mut resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportMirrorMemoryDtcByStatusMask as u8,
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
        let fmt = self.uds_get_number_of_dtcs_by_status_mask(status_mask)
            .map(|x| x.1)
            .unwrap_or(DTCFormatType::Unknown(0));
        let mut result: Vec<DTC> = Vec::new();

        for x in (0..resp.len()).step_by(4) {
            let dtc_code: u32 =
                (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
            let status = resp[x + 3];

            result.push(DTC {
                format: fmt,
                raw: dtc_code,
                status: DTCStatus::Unknown(status), // TODO
                mil_on: status & 0b10000000 != 0,
                readiness_flag: false,
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
    pub fn uds_get_number_of_mirror_memory_dtcs_by_status_mask(
        &mut self,
        status_mask: u8,
    ) -> DiagServerResult<(u8, DTCFormatType, u16)> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportNumberOfMirrorMemoryDtcByStatusMask as u8,
                status_mask,
            ],
        )?;
        if resp.len() != 6 {
            Err(DiagError::InvalidResponseLength)
        } else {
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
    pub fn uds_get_number_of_emissions_related_obd_dtcs_by_status_mask(
        &mut self,
        status_mask: u8,
    ) -> DiagServerResult<(u8, DTCFormatType, u16)> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportNumberOfEmissionsRelatedObdDtcByStatusMask as u8,
                status_mask,
            ],
        )?;
        if resp.len() != 6 {
            Err(DiagError::InvalidResponseLength)
        } else {
            Ok((
                resp[2],
                dtc::dtc_format_from_uds(resp[3]),
                (resp[4] as u16) << 8 | resp[5] as u16,
            ))
        }
    }

    /// Returns a list of OBD emissions related DTCs stored on the ECU
    /// who's status mask matches the provided mask
    pub fn uds_get_emissions_related_obd_dtcs_by_status_mask(
        &mut self,
        status_mask: u8,
    ) -> DiagServerResult<Vec<DTC>> {
        let mut resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportEmissionsRelatedObdDtcByStatusMask as u8,
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
        let fmt = self.uds_get_number_of_dtcs_by_status_mask(status_mask)
            .map(|x| x.1)
            .unwrap_or(DTCFormatType::Unknown(0));
        let mut result: Vec<DTC> = Vec::new();

        for x in (0..resp.len()).step_by(4) {
            let dtc_code: u32 =
                (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
            let status = resp[x + 3];

            result.push(DTC {
                format: fmt,
                raw: dtc_code,
                status: DTCStatus::Unknown(status), // TODO
                mil_on: status & 0b10000000 != 0,
                readiness_flag: false,
            })
        }
        Ok(result)
    }

    ///
    pub fn uds_get_dtc_snapshot_record_by_dtc_number(
        &mut self,
        dtc_mask_record: u32,
        snapshot_record_number: u8,
    ) -> DiagServerResult<u32> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportDtcSnapshotRecordByDtcNumber as u8,
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
    pub fn uds_get_dtc_snapshot_identification(&mut self) -> DiagServerResult<u32> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[DtcSubFunction::ReportDTCSnapshotIdentifier as u8],
        )?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response was: {:02X?}",
            resp
        )))
    }

    /// Returns a list of snapshot records based on the mask of snapshot_record_number (0xFF for all records)
    pub fn uds_get_dtc_snapshot_record_by_record_number(
        &mut self,
        snapshot_record_number: u8,
    ) -> DiagServerResult<u32> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportDtcSnapshotRecordByRecordNumber as u8,
                snapshot_record_number,
            ],
        )?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response was: {:02X?}",
            resp
        )))
    }

    /// Returns the DTCExtendedData record(s) associated with the provided DTC mask and record number.
    /// For the record_number, 0xFE implies all OBD records. and 0xFF implies all records.
    ///
    /// ## Returns
    /// This function will return the ECUs full response if successful
    pub fn uds_get_dtc_extended_data_record_by_dtc_number(
        &mut self,
        dtc: u32,
        extended_data_record_number: u8,
    ) -> DiagServerResult<Vec<u8>> {
        self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportDtcExtendedDataRecordByDtcNumber as u8,
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
    pub fn uds_get_mirror_memory_dtc_extended_data_record_by_dtc_number(
        &mut self,
        dtc: u32,
        extended_data_record_number: u8,
    ) -> DiagServerResult<Vec<u8>> {
        self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportMirrorMemoryDtcExtendedDataRecordByDtcNumber as u8,
                (dtc >> 16) as u8, // High byte
                (dtc >> 8) as u8,  // Mid byte
                dtc as u8,         // Low byte
                extended_data_record_number,
            ],
        )
    }

    /// Returns the number of DTCs stored on the ECU that match the provided severity and status mask
    pub fn uds_get_number_of_dtcs_by_severity_mask_record(
        &mut self,
        severity_mask: u8,
        status_mask: u8,
    ) -> DiagServerResult<u32> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportNumberOfDtcBySeverityMaskRecord as u8,
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
    pub fn uds_get_dtcs_by_severity_mask_record(
        &mut self,
        severity_mask: u8,
        status_mask: u8,
    ) -> DiagServerResult<Vec<DTC>> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportDtcBySeverityMaskRecord as u8,
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
    pub fn uds_get_severity_information_of_dtc(&mut self, dtc: u32) -> DiagServerResult<u32> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[
                DtcSubFunction::ReportSeverityInformationOfDtc as u8,
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
    pub fn uds_get_supported_dtc(&mut self) -> DiagServerResult<Vec<DTC>> {
        let mut resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
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
        let fmt = self.uds_get_number_of_dtcs_by_status_mask(0xFF)
            .map(|x| x.1)
            .unwrap_or(DTCFormatType::Unknown(0));
        let mut result: Vec<DTC> = Vec::new();

        for x in (0..resp.len()).step_by(4) {
            let dtc_code: u32 =
                (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
            let status = resp[x + 3];

            result.push(DTC {
                format: fmt,
                raw: dtc_code,
                status: DTCStatus::Unknown(status), // TODO
                mil_on: status & 0b10000000 != 0,
                readiness_flag: false,
            })
        }
        Ok(result)
    }

    /// Returns the first failed DTC to be detected since the last DTC clear operation
    pub fn uds_get_first_test_failed_dtc(&mut self) -> DiagServerResult<Option<DTC>> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[DtcSubFunction::ReportFirstTestFailedDTC as u8],
        )?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response was: {:02X?}",
            resp
        )))
    }

    /// Returns the first confirmed DTC to be detected since the last DTC clear operation
    pub fn uds_get_first_confirmed_dtc(&mut self) -> DiagServerResult<Option<DTC>> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[DtcSubFunction::ReportFirstConfirmedDTC as u8],
        )?;
        Err(DiagError::NotImplemented(format!(
            "ECU Response was: {:02X?}",
            resp
        )))
    }

    /// Returns the most recent DTC to be detected since the last DTC clear operation
    pub fn uds_get_most_recent_test_failed_dtc(&mut self) -> DiagServerResult<Option<DTC>> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[DtcSubFunction::ReportMostRecentTestFailedDTC as u8],
        )?;
        Err(DiagError::NotImplemented(format!(
            "ReportMostRecentTestFailedDtc ECU Response was: {:02X?}",
            resp
        )))
    }

    /// Returns the most recent DTC to be detected since the last DTC clear operation
    pub fn uds_get_most_recent_confirmed_dtc(&mut self) -> DiagServerResult<Option<DTC>> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[DtcSubFunction::ReportMostRecentConfirmedDTC as u8],
        )?;
        Err(DiagError::NotImplemented(format!(
            "ReportMostRecentConfirmedDtc ECU Response was: {:02X?}",
            resp
        )))
    }

    /// Returns the current number of 'pre-failed' DTCs on the ECU, which have not yet been confirmed
    /// as being either 'pending' or 'confirmed'
    ///
    /// ## Returns
    /// This function will return a vector of information, where each element is a tuple containing the following values:
    /// 1. (u32) - DTC Code
    /// 2. (u8) - Fault detection counter
    pub fn uds_get_dtc_fault_detection_counter(&mut self) -> DiagServerResult<Vec<(u32, u8)>> {
        let mut resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
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
            let dtc_code: u32 =
                (resp[x] as u32) << 16 | (resp[x + 1] as u32) << 8 | resp[x + 2] as u32;
            result.push((dtc_code, resp[x + 3]))
        }
        Ok(result)
    }

    /// Returns a list of DTCs that have a permanent status
    pub fn uds_get_dtc_with_permanent_status(&mut self) -> DiagServerResult<Vec<DTC>> {
        let resp = self.send_command_with_response(
            auto_uds::Command::ReadDTCInformation,
            &[DtcSubFunction::ReportDTCWithPermanentStatus as u8],
        )?;
        Err(DiagError::NotImplemented(format!(
            "ReportDtcWithPermanentStatus ECU Response was: {:02X?}",
            resp
        )))
    }
}
