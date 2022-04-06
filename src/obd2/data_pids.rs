use crate::obd2::{
    CommandedSecondaryAirStatus, Distance, FuelSystemStatus, OBD2DiagnosticServer, OBDStandard,
    ObdEnumValue, ObdUnitType, ObdValue, Pressure, Speed, Temperature, Time,
};
use crate::{DiagError, DiagServerResult, DiagnosticServer};
use strum_macros::EnumString;

use super::FuelTypeCoding;

/// OBD2 data PIDs used for Service 01 and 02
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, EnumString)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum DataPid {
    PidSupport0120,
    StatusSinceDTCCleared,
    FreezeDTC,
    FuelSystemStatus,
    CalculatedEngineLoad,
    EngineCoolantTemp,
    ShortTermFuelTrimBank1,
    LongTermFuelTrimBank1,
    ShortTermFuelTrimBank2,
    LongTermFuelTrimBank2,
    FuelPressureGauge,
    IntakeManifoldAbsPressure,
    EngineSpeed,
    VehicleSpeed,
    TimingAdvance,
    IntakeAirTemperature,
    MassAirFlow,
    ThrottlePosition,
    CommandedSecondaryAirStatus,
    O2SensorsPresent2Banks,
    OxygenSensor1,
    OxygenSensor2,
    OxygenSensor3,
    OxygenSensor4,
    OxygenSensor5,
    OxygenSensor6,
    OxygenSensor7,
    OxygenSensor8,
    ObdStandard,
    O2SensorsPresent4Banks,
    AuxInputStatus,
    RuntimeSinceStart,
    PidSupport2140,
    MILRuntime,
    FuelRailPressure,
    FuelRailGaugePressure,
    OxygenSensor1LambdaVoltage,
    OxygenSensor2LambdaVoltage,
    OxygenSensor3LambdaVoltage,
    OxygenSensor4LambdaVoltage,
    OxygenSensor5LambdaVoltage,
    OxygenSensor6LambdaVoltage,
    OxygenSensor7LambdaVoltage,
    OxygenSensor8LambdaVoltage,
    CommandedEGR,
    EGRError,
    CommandedEvapPurge,
    FuelTankLevelInput,
    WarmupsSinceCodesCleared,
    DistanceTraveledSinceCodesCleared,
    EvapSystemVaporPressure,
    AbsBarometricPressure,
    OxygenSensor1LambdaCurrent,
    OxygenSensor2LambdaCurrent,
    OxygenSensor3LambdaCurrent,
    OxygenSensor4LambdaCurrent,
    OxygenSensor5LambdaCurrent,
    OxygenSensor6LambdaCurrent,
    OxygenSensor7LambdaCurrent,
    OxygenSensor8LambdaCurrent,
    CatTempBank1Sensor1,
    CatTempBank2Sensor1,
    CatTempBank1Sensor2,
    CatTempBank2Sensor2,
    PidSupport4160,
    MonitorStatusDriveCycle,
    ControlModuleVoltage,
    AbsLoadValue,
    CommandedLambda,
    RelativeThrottlePosition,
    AmbientAirTemp,
    AbsoluteThrottlePositionB,
    AbsoluteThrottlePositionC,
    AbsoluteThrottlePositionD,
    AbsoluteThrottlePositionE,
    AbsoluteThrottlePositionF,
    CommandedThrottleActuator,
    TimeRunSinceMILOn,
    TimeSinceCodesCleared,
    MaximumLambdaVoltageCurrentPressure,
    MaximumAirFlowRate,
    FuelType,
    EthanolFuelPercentage,
    AbsoluteEvapSystemVaporPressure,
    EvapSystemVaporPressure2,
    ShortTermSecondaryOxygenSensorTrimBank1and3,
    LongTermSecondaryOxygenSensorTrimBank1and3,
    ShortTermSecondaryOxygenSensorTrimBank2and4,
    LongTermSecondaryOxygenSensorTrimBank2and4,
    FuelRailAbsPressure,
    RelativePedalPosition,
    HybridBatteryPackLife,
    EngineOilTemp,
    FuelInjectionTiming,
    EngineFuelRate,
    EmissionsStandard,
    PidSupport6180,
    DriverDemandTorquePercent,
    EngineTorquePercent,
    EngineTorqueData,
    AuxInputOutputSupport,
    MassAirFlowSensor2,
    EngineCoolantTemp2,
    IntakeAirTemp2,
    Unknown(u8),
}

impl From<u8> for DataPid {
    fn from(x: u8) -> Self {
        match x {
            0x00 => DataPid::PidSupport0120,
            0x01 => DataPid::StatusSinceDTCCleared,
            0x02 => DataPid::FreezeDTC,
            0x03 => DataPid::FuelSystemStatus,
            0x04 => DataPid::CalculatedEngineLoad,
            0x05 => DataPid::EngineCoolantTemp,
            0x06 => DataPid::ShortTermFuelTrimBank1,
            0x07 => DataPid::LongTermFuelTrimBank1,
            0x08 => DataPid::ShortTermFuelTrimBank2,
            0x09 => DataPid::LongTermFuelTrimBank2,
            0x0A => DataPid::FuelPressureGauge,
            0x0B => DataPid::IntakeManifoldAbsPressure,
            0x0C => DataPid::EngineSpeed,
            0x0D => DataPid::VehicleSpeed,
            0x0E => DataPid::TimingAdvance,
            0x0F => DataPid::IntakeAirTemperature,
            0x10 => DataPid::MassAirFlow,
            0x11 => DataPid::ThrottlePosition,
            0x12 => DataPid::CommandedSecondaryAirStatus,
            0x13 => DataPid::O2SensorsPresent2Banks,
            0x14 => DataPid::OxygenSensor1,
            0x15 => DataPid::OxygenSensor2,
            0x16 => DataPid::OxygenSensor3,
            0x17 => DataPid::OxygenSensor4,
            0x18 => DataPid::OxygenSensor5,
            0x19 => DataPid::OxygenSensor6,
            0x1A => DataPid::OxygenSensor7,
            0x1B => DataPid::OxygenSensor8,
            0x1C => DataPid::ObdStandard,
            0x1D => DataPid::O2SensorsPresent4Banks,
            0x1E => DataPid::AuxInputStatus,
            0x1F => DataPid::RuntimeSinceStart,
            0x20 => DataPid::PidSupport2140,
            0x21 => DataPid::MILRuntime,
            0x22 => DataPid::FuelRailPressure,
            0x23 => DataPid::FuelRailGaugePressure,
            0x24 => DataPid::OxygenSensor1LambdaVoltage,
            0x25 => DataPid::OxygenSensor2LambdaVoltage,
            0x26 => DataPid::OxygenSensor3LambdaVoltage,
            0x27 => DataPid::OxygenSensor4LambdaVoltage,
            0x28 => DataPid::OxygenSensor5LambdaVoltage,
            0x29 => DataPid::OxygenSensor6LambdaVoltage,
            0x2A => DataPid::OxygenSensor7LambdaVoltage,
            0x2B => DataPid::OxygenSensor8LambdaVoltage,
            0x2C => DataPid::CommandedEGR,
            0x2D => DataPid::EGRError,
            0x2E => DataPid::CommandedEvapPurge,
            0x2F => DataPid::FuelTankLevelInput,
            0x30 => DataPid::WarmupsSinceCodesCleared,
            0x31 => DataPid::DistanceTraveledSinceCodesCleared,
            0x32 => DataPid::EvapSystemVaporPressure,
            0x33 => DataPid::AbsBarometricPressure,
            0x34 => DataPid::OxygenSensor1LambdaCurrent,
            0x35 => DataPid::OxygenSensor2LambdaCurrent,
            0x36 => DataPid::OxygenSensor3LambdaCurrent,
            0x37 => DataPid::OxygenSensor4LambdaCurrent,
            0x38 => DataPid::OxygenSensor5LambdaCurrent,
            0x39 => DataPid::OxygenSensor6LambdaCurrent,
            0x3A => DataPid::OxygenSensor7LambdaCurrent,
            0x3B => DataPid::OxygenSensor8LambdaCurrent,
            0x3C => DataPid::CatTempBank1Sensor1,
            0x3D => DataPid::CatTempBank2Sensor1,
            0x3E => DataPid::CatTempBank1Sensor2,
            0x3F => DataPid::CatTempBank2Sensor2,
            0x40 => DataPid::PidSupport4160,
            0x41 => DataPid::MonitorStatusDriveCycle,
            0x42 => DataPid::ControlModuleVoltage,
            0x43 => DataPid::AbsLoadValue,
            0x44 => DataPid::CommandedLambda,
            0x45 => DataPid::RelativeThrottlePosition,
            0x46 => DataPid::AmbientAirTemp,
            0x47 => DataPid::AbsoluteThrottlePositionB,
            0x48 => DataPid::AbsoluteThrottlePositionC,
            0x49 => DataPid::AbsoluteThrottlePositionD,
            0x4A => DataPid::AbsoluteThrottlePositionE,
            0x4B => DataPid::AbsoluteThrottlePositionF,
            0x4C => DataPid::CommandedThrottleActuator,
            0x4D => DataPid::TimeRunSinceMILOn,
            0x4E => DataPid::TimeSinceCodesCleared,
            0x4F => DataPid::MaximumLambdaVoltageCurrentPressure,
            0x50 => DataPid::MaximumAirFlowRate,
            0x51 => DataPid::FuelType,
            0x52 => DataPid::EthanolFuelPercentage,
            0x53 => DataPid::AbsoluteEvapSystemVaporPressure,
            0x54 => DataPid::EvapSystemVaporPressure2,
            0x55 => DataPid::ShortTermSecondaryOxygenSensorTrimBank1and3,
            0x56 => DataPid::LongTermSecondaryOxygenSensorTrimBank1and3,
            0x57 => DataPid::ShortTermSecondaryOxygenSensorTrimBank2and4,
            0x58 => DataPid::LongTermSecondaryOxygenSensorTrimBank2and4,
            0x59 => DataPid::FuelRailAbsPressure,
            0x5A => DataPid::RelativePedalPosition,
            0x5B => DataPid::HybridBatteryPackLife,
            0x5C => DataPid::EngineOilTemp,
            0x5D => DataPid::FuelInjectionTiming,
            0x5E => DataPid::EngineFuelRate,
            0x5F => DataPid::EmissionsStandard,
            0x60 => DataPid::PidSupport6180,
            0x61 => DataPid::DriverDemandTorquePercent,
            0x62 => DataPid::EngineTorquePercent,
            0x63 => DataPid::EngineTorqueData,
            0x64 => DataPid::AuxInputOutputSupport,
            0x65 => DataPid::MassAirFlowSensor2,
            0x66 => DataPid::EngineCoolantTemp2,
            0x67 => DataPid::IntakeAirTemp2,
            _ => DataPid::Unknown(x),
        }
    }
}

impl From<DataPid> for u8 {
    fn from(x: DataPid) -> Self {
        match x {
            DataPid::PidSupport0120 => 0x00,
            DataPid::StatusSinceDTCCleared => 0x01,
            DataPid::FreezeDTC => 0x02,
            DataPid::FuelSystemStatus => 0x03,
            DataPid::CalculatedEngineLoad => 0x04,
            DataPid::EngineCoolantTemp => 0x05,
            DataPid::ShortTermFuelTrimBank1 => 0x06,
            DataPid::LongTermFuelTrimBank1 => 0x07,
            DataPid::ShortTermFuelTrimBank2 => 0x08,
            DataPid::LongTermFuelTrimBank2 => 0x09,
            DataPid::FuelPressureGauge => 0x0A,
            DataPid::IntakeManifoldAbsPressure => 0x0B,
            DataPid::EngineSpeed => 0x0C,
            DataPid::VehicleSpeed => 0x0D,
            DataPid::TimingAdvance => 0x0E,
            DataPid::IntakeAirTemperature => 0x0F,
            DataPid::MassAirFlow => 0x10,
            DataPid::ThrottlePosition => 0x11,
            DataPid::CommandedSecondaryAirStatus => 0x12,
            DataPid::O2SensorsPresent2Banks => 0x13,
            DataPid::OxygenSensor1 => 0x14,
            DataPid::OxygenSensor2 => 0x15,
            DataPid::OxygenSensor3 => 0x16,
            DataPid::OxygenSensor4 => 0x17,
            DataPid::OxygenSensor5 => 0x18,
            DataPid::OxygenSensor6 => 0x19,
            DataPid::OxygenSensor7 => 0x1A,
            DataPid::OxygenSensor8 => 0x1B,
            DataPid::ObdStandard => 0x1C,
            DataPid::O2SensorsPresent4Banks => 0x1D,
            DataPid::AuxInputStatus => 0x1E,
            DataPid::RuntimeSinceStart => 0x1F,
            DataPid::PidSupport2140 => 0x20,
            DataPid::MILRuntime => 0x21,
            DataPid::FuelRailPressure => 0x22,
            DataPid::FuelRailGaugePressure => 0x23,
            DataPid::OxygenSensor1LambdaVoltage => 0x24,
            DataPid::OxygenSensor2LambdaVoltage => 0x25,
            DataPid::OxygenSensor3LambdaVoltage => 0x26,
            DataPid::OxygenSensor4LambdaVoltage => 0x27,
            DataPid::OxygenSensor5LambdaVoltage => 0x28,
            DataPid::OxygenSensor6LambdaVoltage => 0x29,
            DataPid::OxygenSensor7LambdaVoltage => 0x2A,
            DataPid::OxygenSensor8LambdaVoltage => 0x2B,
            DataPid::CommandedEGR => 0x2C,
            DataPid::EGRError => 0x2D,
            DataPid::CommandedEvapPurge => 0x2E,
            DataPid::FuelTankLevelInput => 0x2F,
            DataPid::WarmupsSinceCodesCleared => 0x30,
            DataPid::DistanceTraveledSinceCodesCleared => 0x31,
            DataPid::EvapSystemVaporPressure => 0x32,
            DataPid::AbsBarometricPressure => 0x33,
            DataPid::OxygenSensor1LambdaCurrent => 0x34,
            DataPid::OxygenSensor2LambdaCurrent => 0x35,
            DataPid::OxygenSensor3LambdaCurrent => 0x36,
            DataPid::OxygenSensor4LambdaCurrent => 0x37,
            DataPid::OxygenSensor5LambdaCurrent => 0x38,
            DataPid::OxygenSensor6LambdaCurrent => 0x39,
            DataPid::OxygenSensor7LambdaCurrent => 0x3A,
            DataPid::OxygenSensor8LambdaCurrent => 0x3B,
            DataPid::CatTempBank1Sensor1 => 0x3C,
            DataPid::CatTempBank2Sensor1 => 0x3D,
            DataPid::CatTempBank1Sensor2 => 0x3E,
            DataPid::CatTempBank2Sensor2 => 0x3F,
            DataPid::PidSupport4160 => 0x20,
            DataPid::MonitorStatusDriveCycle => 0x41,
            DataPid::ControlModuleVoltage => 0x42,
            DataPid::AbsLoadValue => 0x43,
            DataPid::CommandedLambda => 0x44,
            DataPid::RelativeThrottlePosition => 0x45,
            DataPid::AmbientAirTemp => 0x46,
            DataPid::AbsoluteThrottlePositionB => 0x47,
            DataPid::AbsoluteThrottlePositionC => 0x48,
            DataPid::AbsoluteThrottlePositionD => 0x49,
            DataPid::AbsoluteThrottlePositionE => 0x4A,
            DataPid::AbsoluteThrottlePositionF => 0x4B,
            DataPid::CommandedThrottleActuator => 0x4C,
            DataPid::TimeRunSinceMILOn => 0x4D,
            DataPid::TimeSinceCodesCleared => 0x4E,
            DataPid::MaximumLambdaVoltageCurrentPressure => 0x4F,
            DataPid::MaximumAirFlowRate => 0x50,
            DataPid::FuelType => 0x51,
            DataPid::EthanolFuelPercentage => 0x52,
            DataPid::AbsoluteEvapSystemVaporPressure => 0x53,
            DataPid::EvapSystemVaporPressure2 => 0x54,
            DataPid::ShortTermSecondaryOxygenSensorTrimBank1and3 => 0x55,
            DataPid::LongTermSecondaryOxygenSensorTrimBank1and3 => 0x56,
            DataPid::ShortTermSecondaryOxygenSensorTrimBank2and4 => 0x57,
            DataPid::LongTermSecondaryOxygenSensorTrimBank2and4 => 0x58,
            DataPid::FuelRailAbsPressure => 0x59,
            DataPid::RelativePedalPosition => 0x5A,
            DataPid::HybridBatteryPackLife => 0x5B,
            DataPid::EngineOilTemp => 0x5C,
            DataPid::FuelInjectionTiming => 0x5D,
            DataPid::EngineFuelRate => 0x5E,
            DataPid::EmissionsStandard => 0x5F,
            DataPid::PidSupport6180 => 0x60,
            DataPid::DriverDemandTorquePercent => 0x61,
            DataPid::EngineTorquePercent => 0x62,
            DataPid::EngineTorqueData => 0x63,
            DataPid::AuxInputOutputSupport => 0x64,
            DataPid::MassAirFlowSensor2 => 0x65,
            DataPid::EngineCoolantTemp2 => 0x66,
            DataPid::IntakeAirTemp2 => 0x67,
            DataPid::Unknown(x) => x,
        }
    }
}

impl DataPid {
    fn request_ecu(
        &self,
        server: &mut OBD2DiagnosticServer,
        ff: Option<u16>,
        min_length: usize,
    ) -> DiagServerResult<Vec<u8>> {
        let req = match ff {
            None => vec![0x01, u8::from(*self)],
            Some(ff_id) => vec![0x02, u8::from(*self), (ff_id >> 8) as u8, ff_id as u8],
        };
        let mut r = server.send_byte_array_with_response(&req)?;
        r.drain(0..2);
        if r.len() < min_length {
            return Err(DiagError::InvalidResponseLength);
        }
        return Ok(r);
    }

    /// For value = A*100/255
    fn get_percentage_1_byte(&self, server: &mut OBD2DiagnosticServer, ff: Option<u16>, name: &str) -> DiagServerResult<Vec<ObdValue>> {
        Ok(vec![
            ObdValue::new(
                name,
                ObdUnitType::Percent(
                    self.request_ecu(server, ff, 1)?[0] as f32 * (100.0/255.0)
                )
            )
        ])
    }

    /// Returns parsed value after request the ECU for the PID
    pub(crate) fn get_value(
        &self,
        server: &mut OBD2DiagnosticServer,
        ff: Option<u16>,
    ) -> DiagServerResult<Vec<ObdValue>> {
        match self {
            DataPid::PidSupport0120 => Ok(vec![ObdValue::new(
                "PID support 01-20",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            DataPid::StatusSinceDTCCleared => Err(DiagError::NotImplemented(
                "Status since DTC Cleared unimplemented".into(),
            )),
            DataPid::FreezeDTC => Err(DiagError::NotImplemented("Freeze DTC unimplemented".into())),
            DataPid::FuelSystemStatus => Ok(self
                .request_ecu(server, ff, 1)?
                .iter()
                .enumerate()
                .map(|(idx, byte)| {
                    ObdValue::new(
                        format!("Fuel system status {}", idx + 1),
                        ObdUnitType::Encoded(ObdEnumValue::FuelSystemStatus(
                            FuelSystemStatus::from(*byte),
                        )),
                    )
                })
                .collect()),
            DataPid::CalculatedEngineLoad => self.get_percentage_1_byte(server, ff, "Calculated engine load"),
            DataPid::EngineCoolantTemp => Ok(self
                .request_ecu(server, ff, 1)?
                .iter()
                .map(|x| {
                    ObdValue::new(
                        "Engine coolant temperature",
                        ObdUnitType::Temperature(Temperature::from_celsius(*x as f32 - 40.0)),
                    )
                })
                .collect()),
            DataPid::ShortTermFuelTrimBank1 => Ok(vec![ObdValue::new(
                "Short term fuel trim - Bank 1",
                ObdUnitType::Percent((self.request_ecu(server, ff, 1)?[0] as f32 / 1.28) - 100.0),
            )]),
            DataPid::LongTermFuelTrimBank1 => Ok(vec![ObdValue::new(
                "Long term fuel trim - Bank 1",
                ObdUnitType::Percent((self.request_ecu(server, ff, 1)?[0] as f32 / 1.28) - 100.0),
            )]),
            DataPid::ShortTermFuelTrimBank2 => Ok(vec![ObdValue::new(
                "Short term fuel trim - Bank 2",
                ObdUnitType::Percent((self.request_ecu(server, ff, 1)?[0] as f32 / 1.28) - 100.0),
            )]),
            DataPid::LongTermFuelTrimBank2 => Ok(vec![ObdValue::new(
                "Long term fuel trim - Bank 2",
                ObdUnitType::Percent((self.request_ecu(server, ff, 1)?[0] as f32 / 1.28) - 100.0),
            )]),
            DataPid::FuelPressureGauge => Ok(vec![ObdValue::new(
                "Fuel pressure (gauge pressure)",
                ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                    self.request_ecu(server, ff, 1)?[0] as f32 * 3.0,
                )),
            )]),
            DataPid::IntakeManifoldAbsPressure => Ok(vec![ObdValue::new(
                "Intake manifold absolute pressure",
                ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                    self.request_ecu(server, ff, 1)?[0] as f32,
                )),
            )]),
            DataPid::EngineSpeed => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Engine speed",
                    ObdUnitType::Rpm(((r[0] as u32) << 8) | r[1] as u32),
                )])
            }
            DataPid::VehicleSpeed => Ok(vec![ObdValue::new(
                "Vehicle speed",
                ObdUnitType::Speed(Speed::from_kmh(self.request_ecu(server, ff, 1)?[0] as f32)),
            )]),
            DataPid::TimingAdvance => Ok(vec![ObdValue::new(
                "Timing advance before TDC (degrees)",
                ObdUnitType::Raw(self.request_ecu(server, ff, 1)?[0] as f32 - 64.0),
            )]),
            DataPid::IntakeAirTemperature => Ok(vec![ObdValue::new(
                "Intake air temperature",
                ObdUnitType::Temperature(Temperature::from_celsius(
                    self.request_ecu(server, ff, 1)?[0] as f32 - 40.0,
                )),
            )]),
            DataPid::MassAirFlow => {
                let s = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Mass air flow sensor rate (Grames/sec)",
                    ObdUnitType::Raw(((s[0] as u32) << 8 | s[1] as u32) as f32 / 100.0),
                )])
            }
            DataPid::ThrottlePosition => self.get_percentage_1_byte(server, ff, "Throttle position"),
            DataPid::CommandedSecondaryAirStatus => Ok(vec![ObdValue::new(
                "Commanded secondary air status",
                ObdUnitType::Encoded(ObdEnumValue::CommandedAirStatus(
                    CommandedSecondaryAirStatus::from(self.request_ecu(server, ff, 1)?[0]),
                )),
            )]),
            DataPid::O2SensorsPresent2Banks => Ok(vec![ObdValue::new(
                "Oxygen sensors present in 2 banks",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 1)?),
            )]),
            DataPid::OxygenSensor1 => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 1 voltage",
                        ObdUnitType::Volts(r[0] as f32 / 200.0),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 1 short term fuel trim",
                        ObdUnitType::Percent((r[1] as f32 / 1.28) - 100.0),
                    ),
                ])
            }
            DataPid::OxygenSensor2 => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 2 voltage",
                        ObdUnitType::Volts(r[0] as f32 / 200.0),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 2 short term fuel trim",
                        ObdUnitType::Percent((r[1] as f32 / 1.28) - 100.0),
                    ),
                ])
            }
            DataPid::OxygenSensor3 => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 3 voltage",
                        ObdUnitType::Volts(r[0] as f32 / 200.0),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 3 short term fuel trim",
                        ObdUnitType::Percent((r[1] as f32 / 1.28) - 100.0),
                    ),
                ])
            }
            DataPid::OxygenSensor4 => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 4 voltage",
                        ObdUnitType::Volts(r[0] as f32 / 200.0),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 4 short term fuel trim",
                        ObdUnitType::Percent((r[1] as f32 / 1.28) - 100.0),
                    ),
                ])
            }
            DataPid::OxygenSensor5 => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 5 voltage",
                        ObdUnitType::Volts(r[0] as f32 / 200.0),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 5 short term fuel trim",
                        ObdUnitType::Percent((r[1] as f32 / 1.28) - 100.0),
                    ),
                ])
            }
            DataPid::OxygenSensor6 => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 6 voltage",
                        ObdUnitType::Volts(r[0] as f32 / 200.0),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 6 short term fuel trim",
                        ObdUnitType::Percent((r[1] as f32 / 1.28) - 100.0),
                    ),
                ])
            }
            DataPid::OxygenSensor7 => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 7 voltage",
                        ObdUnitType::Volts(r[0] as f32 / 200.0),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 7 short term fuel trim",
                        ObdUnitType::Percent((r[1] as f32 / 1.28) - 100.0),
                    ),
                ])
            }
            DataPid::OxygenSensor8 => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 8 voltage",
                        ObdUnitType::Volts(r[0] as f32 / 200.0),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 8 short term fuel trim",
                        ObdUnitType::Percent((r[1] as f32 / 1.28) - 100.0),
                    ),
                ])
            }
            DataPid::ObdStandard => Ok(vec![ObdValue::new(
                "OBD Standard",
                ObdUnitType::Encoded(ObdEnumValue::ObdStandard(OBDStandard::from(
                    self.request_ecu(server, ff, 1)?[0],
                ))),
            )]),
            DataPid::O2SensorsPresent4Banks => Ok(vec![ObdValue::new(
                "Oxygen sensors present in 4 banks",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 1)?),
            )]),

            //DataPid::AuxInputStatus => {}
            DataPid::RuntimeSinceStart => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Runtime since engine start",
                    ObdUnitType::Time(Time::from_seconds(
                        ((r[0] as u32) << 8 | r[1] as u32) as f32,
                    )),
                )])
            }
            DataPid::PidSupport2140 => Ok(vec![ObdValue::new(
                "PID support 21-40",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            DataPid::MILRuntime => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Distance travelled with MIL on",
                    ObdUnitType::Distance(Distance::from_kilometers(
                        ((r[0] as u32) << 8 | r[1] as u32) as f32,
                    )),
                )])
            }
            DataPid::FuelRailPressure => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Fuel rail pressure (Relative to manifold vacuum)",
                    ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                        ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.079,
                    )),
                )])
            }
            DataPid::FuelRailGaugePressure => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Fuel rail gauge pressure",
                    ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                        ((r[0] as u32) << 8 | r[1] as u32) as f32 * 10.0,
                    )),
                )])
            }
            DataPid::OxygenSensor1LambdaVoltage => {
                let r = self.request_ecu(server, ff, 4)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 1 Lambda",
                        ObdUnitType::Raw(
                            ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.000030517578125,
                        ),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 1 voltage",
                        ObdUnitType::Volts(
                            ((r[2] as u32) << 8 | r[3] as u32) as f32 * 0.0001220703125,
                        ),
                    ),
                ])
            }
            DataPid::OxygenSensor2LambdaVoltage => {
                let r = self.request_ecu(server, ff, 4)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 2 Lambda",
                        ObdUnitType::Raw(
                            ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.000030517578125,
                        ),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 2 voltage",
                        ObdUnitType::Volts(
                            ((r[2] as u32) << 8 | r[3] as u32) as f32 * 0.0001220703125,
                        ),
                    ),
                ])
            }
            DataPid::OxygenSensor3LambdaVoltage => {
                let r = self.request_ecu(server, ff, 4)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 3 Lambda",
                        ObdUnitType::Raw(
                            ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.000030517578125,
                        ),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 3 voltage",
                        ObdUnitType::Volts(
                            ((r[2] as u32) << 8 | r[3] as u32) as f32 * 0.0001220703125,
                        ),
                    ),
                ])
            }
            DataPid::OxygenSensor4LambdaVoltage => {
                let r = self.request_ecu(server, ff, 4)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 4 Lambda",
                        ObdUnitType::Raw(
                            ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.000030517578125,
                        ),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 4 voltage",
                        ObdUnitType::Volts(
                            ((r[2] as u32) << 8 | r[3] as u32) as f32 * 0.0001220703125,
                        ),
                    ),
                ])
            }
            DataPid::OxygenSensor5LambdaVoltage => {
                let r = self.request_ecu(server, ff, 4)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 5 Lambda",
                        ObdUnitType::Raw(
                            ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.000030517578125,
                        ),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 5 voltage",
                        ObdUnitType::Volts(
                            ((r[2] as u32) << 8 | r[3] as u32) as f32 * 0.0001220703125,
                        ),
                    ),
                ])
            }
            DataPid::OxygenSensor6LambdaVoltage => {
                let r = self.request_ecu(server, ff, 4)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 6 Lambda",
                        ObdUnitType::Raw(
                            ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.000030517578125,
                        ),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 6 voltage",
                        ObdUnitType::Volts(
                            ((r[2] as u32) << 8 | r[3] as u32) as f32 * 0.0001220703125,
                        ),
                    ),
                ])
            }
            DataPid::OxygenSensor7LambdaVoltage => {
                let r = self.request_ecu(server, ff, 4)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 7 Lambda",
                        ObdUnitType::Raw(
                            ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.000030517578125,
                        ),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 7 voltage",
                        ObdUnitType::Volts(
                            ((r[2] as u32) << 8 | r[3] as u32) as f32 * 0.0001220703125,
                        ),
                    ),
                ])
            }
            DataPid::OxygenSensor8LambdaVoltage => {
                let r = self.request_ecu(server, ff, 4)?;
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 8 Lambda",
                        ObdUnitType::Raw(
                            ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.000030517578125,
                        ),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 8 voltage",
                        ObdUnitType::Volts(
                            ((r[2] as u32) << 8 | r[3] as u32) as f32 * 0.0001220703125,
                        ),
                    ),
                ])
            }
            DataPid::CommandedEGR => self.get_percentage_1_byte(server, ff, "Commanded EGR"),
            DataPid::EGRError => Ok(vec![
                ObdValue::new(
                    "EGR Error",
                    ObdUnitType::Percent(
                        self.request_ecu(server, ff, 1)?[0] as f32 * (100.0/128.0) - 100.0,
                    ),
                )
            ]),
            DataPid::CommandedEvapPurge => self.get_percentage_1_byte(server, ff, "Commanded evaporative purge"),
            DataPid::FuelTankLevelInput => self.get_percentage_1_byte(server, ff, "Fuel tank level input"),
            DataPid::WarmupsSinceCodesCleared => Ok(vec![
                ObdValue::new(
                    "Warm-ups since codes cleared",
                    ObdUnitType::Raw(
                        self.request_ecu(server, ff, 1)?[0] as f32,
                    ),
                )
            ]),
            DataPid::DistanceTraveledSinceCodesCleared => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![
                    ObdValue::new(
                        "Distance traveled since codes cleared",
                        ObdUnitType::Distance(
                            Distance::from_kilometers(
                                (((r[0] as u32) << 8) | r[1] as u32) as f32
                            )
                        ),
                    )
                ])
            }
            DataPid::EvapSystemVaporPressure => {
                let r: Vec<i32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as i32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Evaporative System Vapor Pressure",
                        ObdUnitType::Pressure(
                            Pressure::from_kilo_pascal(
                                (r[0] << 8 | r[1]) as f32 / 4000.0
                            )
                        ),
                    )
                ])
            }
            DataPid::AbsBarometricPressure => Ok(vec![
                ObdValue::new(
                    "Absolute Barometric Pressure",
                    ObdUnitType::Pressure(
                        Pressure::from_kilo_pascal(
                            self.request_ecu(server, ff, 1)?[0] as f32
                        )
                    ),
                )
            ]),
            DataPid::OxygenSensor1LambdaCurrent => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 1 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0/65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 1 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            DataPid::OxygenSensor2LambdaCurrent => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 2 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0/65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 2 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            DataPid::OxygenSensor3LambdaCurrent => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 3 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0/65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 3 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            DataPid::OxygenSensor4LambdaCurrent => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 4 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0/65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 4 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            DataPid::OxygenSensor5LambdaCurrent => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 5 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0/65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 5 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            DataPid::OxygenSensor6LambdaCurrent => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 6 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0/65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 6 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            DataPid::OxygenSensor7LambdaCurrent => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 7 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0/65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 7 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            DataPid::OxygenSensor8LambdaCurrent => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 8 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0/65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 8 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            DataPid::CatTempBank1Sensor1 => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Catalyst Temperature bank 1, sensor 1",
                        ObdUnitType::Temperature(
                            Temperature::from_celsius(
                                ((r[0] << 8 | r[1]) as f32 / 10.0) - 40.0
                            )
                        ),
                    )
                ])
            }
            DataPid::CatTempBank2Sensor1 => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Catalyst Temperature bank 2, sensor 1",
                        ObdUnitType::Temperature(
                            Temperature::from_celsius(
                                ((r[0] << 8 | r[1]) as f32 / 10.0) - 40.0
                            )
                        ),
                    )
                ])
            }
            DataPid::CatTempBank1Sensor2 => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Catalyst Temperature bank 1, sensor 2",
                        ObdUnitType::Temperature(
                            Temperature::from_celsius(
                                ((r[0] << 8 | r[1]) as f32 / 10.0) - 40.0
                            )
                        ),
                    )
                ])
            }
            DataPid::CatTempBank2Sensor2 => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Catalyst Temperature bank 2, sensor 2",
                        ObdUnitType::Temperature(
                            Temperature::from_celsius(
                                ((r[0] << 8 | r[1]) as f32 / 10.0) - 40.0
                            )
                        ),
                    )
                ])
            }
            DataPid::PidSupport4160 => Ok(vec![ObdValue::new(
                "PID support 41-60",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            DataPid::MonitorStatusDriveCycle => Ok(vec![ObdValue::new(
                "Monitor status this drive cycle",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            DataPid::ControlModuleVoltage => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Control module voltage",
                        ObdUnitType::Volts((r[0] << 8 | r[1]) as f32 / 1000.0)
                    )
                ])
            }
            DataPid::AbsLoadValue => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Absolute load value",
                        ObdUnitType::Percent(
                            (r[0] << 8 | r[1]) as f32 * (100.0/255.0)
                        )
                    )
                ])
            }
            DataPid::CommandedLambda => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Commanded Lambda",
                        ObdUnitType::Raw(
                            (r[0] << 8 | r[1]) as f32 * (2.0/65536.0)
                        )
                    )
                ])
            }
            DataPid::RelativeThrottlePosition => self.get_percentage_1_byte(server, ff, "Relative throttle position"),
            DataPid::AmbientAirTemp => Ok(vec![
                ObdValue::new(
                    "Ambient air temperature",
                    ObdUnitType::Temperature(
                        Temperature::from_celsius(
                            self.request_ecu(server, ff, 1)?[0] as f32 - 40.0
                        )
                    )
                )
            ]),
            DataPid::AbsoluteThrottlePositionB => self.get_percentage_1_byte(server, ff, "Absolute throttle position B"),
            DataPid::AbsoluteThrottlePositionC => self.get_percentage_1_byte(server, ff, "Absolute throttle position C"),
            DataPid::AbsoluteThrottlePositionD => self.get_percentage_1_byte(server, ff, "Absolute throttle position D"),
            DataPid::AbsoluteThrottlePositionE => self.get_percentage_1_byte(server, ff, "Absolute throttle position E"),
            DataPid::AbsoluteThrottlePositionF => self.get_percentage_1_byte(server, ff, "Absolute throttle position F"),
            DataPid::CommandedThrottleActuator => self.get_percentage_1_byte(server, ff, "Commanded throttle actuator"),
            DataPid::TimeRunSinceMILOn => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Time run with MIL on",
                        ObdUnitType::Time(
                            Time::from_seconds(
                                (r[0] << 8 | r[1]) as f32 * 60.0
                            )
                        )
                    )
                ])
            }
            DataPid::TimeSinceCodesCleared => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Time since trouble codes cleared",
                        ObdUnitType::Time(
                            Time::from_seconds(
                                (r[0] << 8 | r[1]) as f32 * 60.0
                            )
                        )
                    )
                ])
            }
            DataPid::MaximumLambdaVoltageCurrentPressure => {
                let r: Vec<u32> = self.request_ecu(server, ff, 4)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Maximum value for Lambda",
                        ObdUnitType::Raw(r[0] as f32)
                    ),
                    ObdValue::new(
                        "Maximum value for oxygen sensor voltage",
                        ObdUnitType::Volts(r[1] as f32)
                    ),
                    ObdValue::new(
                        "Maximum value oxygen sensor current",
                        ObdUnitType::Raw(r[2] as f32)
                    ),
                    ObdValue::new(
                        "Maximum value for intake manifold absolute pressure",
                        ObdUnitType::Pressure(
                            Pressure::from_kilo_pascal(
                                r[3] as f32 * 10.0
                            )
                        )
                    )
                ])
            }
            DataPid::MaximumAirFlowRate => {
                let r: Vec<u32> = self.request_ecu(server, ff, 1)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Maximum value for air flow rate from mass air flow sensor",
                        ObdUnitType::Raw(r[0] as f32 * 10.0)
                    )
                ])
            }
            DataPid::FuelType => {
                Ok(vec![
                    ObdValue::new(
                        "Fuel type",
                        ObdUnitType::Encoded(
                            ObdEnumValue::FuelType(
                                FuelTypeCoding::from(self.request_ecu(server, ff, 1)?[0])
                            )
                        )
                    )
                ])
            }
            DataPid::EthanolFuelPercentage => self.get_percentage_1_byte(server, ff, "Ethanol fuel"),
            DataPid::AbsoluteEvapSystemVaporPressure => {
                let r: Vec<i32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as i32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Absolute evaporative system vapor pressure",
                        ObdUnitType::Pressure(
                            Pressure::from_kilo_pascal(
                                (r[0] << 8 | r[1]) as f32 / 200.0
                            )
                        )
                    )
                ])
            }
            DataPid::EvapSystemVaporPressure2 => {
                let r: Vec<i32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as i32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Evaporative system vapor pressure",
                        ObdUnitType::Pressure(
                            Pressure::from_kilo_pascal(
                                (r[0] << 8 | r[1]) as f32 / 1000.0
                            )
                        )
                    )
                ])
            }
            DataPid::ShortTermSecondaryOxygenSensorTrimBank1and3 => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Short term secondary oxygen sensor trim bank 1",
                        ObdUnitType::Percent(
                            r[0] as f32 * (100.0/128.0) - 100.0
                        )
                    ),
                    ObdValue::new(
                        "Short term secondary oxygen sensor trim bank 3",
                        ObdUnitType::Percent(
                            r[1] as f32 * (100.0/128.0) - 100.0
                        )
                    )
                ])
            }
            DataPid::LongTermSecondaryOxygenSensorTrimBank1and3 => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Long term secondary oxygen sensor trim bank 1",
                        ObdUnitType::Percent(
                            r[0] as f32 * (100.0/128.0) - 100.0
                        )
                    ),
                    ObdValue::new(
                        "Long term secondary oxygen sensor trim bank 3",
                        ObdUnitType::Percent(
                            r[1] as f32 * (100.0/128.0) - 100.0
                        )
                    )
                ])
            }
            DataPid::ShortTermSecondaryOxygenSensorTrimBank2and4 => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Short term secondary oxygen sensor trim bank 2",
                        ObdUnitType::Percent(
                            r[0] as f32 * (100.0/128.0) - 100.0
                        )
                    ),
                    ObdValue::new(
                        "Short term secondary oxygen sensor trim bank 4",
                        ObdUnitType::Percent(
                            r[1] as f32 * (100.0/128.0) - 100.0
                        )
                    )
                ])
            }
            DataPid::LongTermSecondaryOxygenSensorTrimBank2and4 => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Long term secondary oxygen sensor trim bank 2",
                        ObdUnitType::Percent(
                            r[0] as f32 * (100.0/128.0) - 100.0
                        )
                    ),
                    ObdValue::new(
                        "Long term secondary oxygen sensor trim bank 4",
                        ObdUnitType::Percent(
                            r[1] as f32 * (100.0/128.0) - 100.0
                        )
                    )
                ])
            }
            DataPid::FuelRailAbsPressure => {
                let r: Vec<u32> = self.request_ecu(server, ff, 2)?.iter().map(|x| *x as u32).collect();
                Ok(vec![
                    ObdValue::new(
                        "Fuel rail absolute pressure",
                        ObdUnitType::Pressure(
                            Pressure::from_kilo_pascal(
                                (r[0] << 8 | r[1]) as f32 * 10.0
                            )
                        )
                    )
                ])
            }
            DataPid::RelativePedalPosition => self.get_percentage_1_byte(server, ff, "Relative accelerator pedal position"),
            DataPid::HybridBatteryPackLife => self.get_percentage_1_byte(server, ff, "Hybrid battery pack remaining life"),
            /*
            DataPid::EngineOilTemp => {}
            DataPid::FuelInjectionTiming => {}
            DataPid::EngineFuelRate => {}
            */
            DataPid::EmissionsStandard => Ok(vec![ObdValue::new(
                "Emission requirments to which the vehicle is designed",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 1)?),
            )]),
            DataPid::PidSupport6180 => Ok(vec![ObdValue::new(
                "PID support 61-80",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            /*
            DataPid::DriverDemandTorquePercent => {}
            DataPid::EngineTorquePercent => {}
            DataPid::EngineTorqueData => {}
            DataPid::AuxInputOutputSupport => {}
            DataPid::MassAirFlowSensor2 => {}
            DataPid::EngineCoolantTemp2 => {}
            DataPid::IntakeAirTemp2 => {}
            DataPid::Unknown(_) => {}
             */
            _ => Err(DiagError::NotImplemented(format!("Parsing {:02X?}", self))),
        }
    }
}
