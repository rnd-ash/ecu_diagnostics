use crate::dynamic_diag::DynamicDiagSession;
use crate::obd2::{
    Distance, ObdEnumValue, ObdUnitType, ObdValue, Pressure, Speed, Temperature, Time,
};
use crate::{DiagError, DiagServerResult};
use auto_uds::obd2::{
    CommandedSecondaryAirStatusByte, DataPidByte, FuelSystemStatusByte, FuelTypeCodingByte,
    ObdStandardByte,
};
use auto_uds::ByteWrapper::Standard;
// use strum_macros::EnumString;

/// Data PID wrapper
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DataPidWrapper(DataPidByte);

impl DataPidWrapper {
    fn request_ecu(
        &self,
        server: &mut DynamicDiagSession,
        ff: Option<u16>,
        min_length: usize,
    ) -> DiagServerResult<Vec<u8>> {
        let req = match ff {
            None => vec![0x01, self.0.into()],
            Some(ff_id) => vec![0x02, self.0.into(), (ff_id >> 8) as u8, ff_id as u8],
        };
        let mut r = server.send_byte_array_with_response(&req)?;
        r.drain(0..2);
        if r.len() < min_length {
            return Err(DiagError::InvalidResponseLength);
        }
        Ok(r)
    }

    /// For value = A*100/255
    fn get_percentage_1_byte(
        &self,
        server: &mut DynamicDiagSession,
        ff: Option<u16>,
        name: &str,
    ) -> DiagServerResult<Vec<ObdValue>> {
        Ok(vec![ObdValue::new(
            name,
            ObdUnitType::Percent(self.request_ecu(server, ff, 1)?[0] as f32 * (100.0 / 255.0)),
        )])
    }

    /// Returns parsed value after request the ECU for the PID
    #[allow(clippy::excessive_precision)]
    pub(crate) fn get_value(
        &self,
        server: &mut DynamicDiagSession,
        ff: Option<u16>,
    ) -> DiagServerResult<Vec<ObdValue>> {
        use auto_uds::obd2::DataPid::*;

        match self.0 {
            Standard(PidSupport0120) => Ok(vec![ObdValue::new(
                "PID support 01-20",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            Standard(StatusSinceDTCCleared) => Err(DiagError::NotImplemented(
                "Status since DTC Cleared unimplemented".into(),
            )),
            Standard(FreezeDTC) => {
                Err(DiagError::NotImplemented("Freeze DTC unimplemented".into()))
            }
            Standard(FuelSystemStatus) => Ok(self
                .request_ecu(server, ff, 1)?
                .iter()
                .enumerate()
                .map(|(idx, byte)| {
                    ObdValue::new(
                        format!("Fuel system status {}", idx + 1),
                        ObdUnitType::Encoded(ObdEnumValue::FuelSystemStatus(
                            FuelSystemStatusByte::from(*byte),
                        )),
                    )
                })
                .collect()),
            Standard(CalculatedEngineLoad) => {
                self.get_percentage_1_byte(server, ff, "Calculated engine load")
            }
            Standard(EngineCoolantTemp) => Ok(self
                .request_ecu(server, ff, 1)?
                .iter()
                .map(|x| {
                    ObdValue::new(
                        "Engine coolant temperature",
                        ObdUnitType::Temperature(Temperature::from_celsius(*x as f32 - 40.0)),
                    )
                })
                .collect()),
            Standard(ShortTermFuelTrimBank1) => Ok(vec![ObdValue::new(
                "Short term fuel trim - Bank 1",
                ObdUnitType::Percent((self.request_ecu(server, ff, 1)?[0] as f32 / 1.28) - 100.0),
            )]),
            Standard(LongTermFuelTrimBank1) => Ok(vec![ObdValue::new(
                "Long term fuel trim - Bank 1",
                ObdUnitType::Percent((self.request_ecu(server, ff, 1)?[0] as f32 / 1.28) - 100.0),
            )]),
            Standard(ShortTermFuelTrimBank2) => Ok(vec![ObdValue::new(
                "Short term fuel trim - Bank 2",
                ObdUnitType::Percent((self.request_ecu(server, ff, 1)?[0] as f32 / 1.28) - 100.0),
            )]),
            Standard(LongTermFuelTrimBank2) => Ok(vec![ObdValue::new(
                "Long term fuel trim - Bank 2",
                ObdUnitType::Percent((self.request_ecu(server, ff, 1)?[0] as f32 / 1.28) - 100.0),
            )]),
            Standard(FuelPressureGauge) => Ok(vec![ObdValue::new(
                "Fuel pressure (gauge pressure)",
                ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                    self.request_ecu(server, ff, 1)?[0] as f32 * 3.0,
                )),
            )]),
            Standard(IntakeManifoldAbsPressure) => Ok(vec![ObdValue::new(
                "Intake manifold absolute pressure",
                ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                    self.request_ecu(server, ff, 1)?[0] as f32,
                )),
            )]),
            Standard(EngineSpeed) => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Engine speed",
                    ObdUnitType::Rpm(((r[0] as u32) << 8) | r[1] as u32),
                )])
            }
            Standard(VehicleSpeed) => Ok(vec![ObdValue::new(
                "Vehicle speed",
                ObdUnitType::Speed(Speed::from_kmh(self.request_ecu(server, ff, 1)?[0] as f32)),
            )]),
            Standard(TimingAdvance) => Ok(vec![ObdValue::new(
                "Timing advance before TDC (degrees)",
                ObdUnitType::Raw(self.request_ecu(server, ff, 1)?[0] as f32 - 64.0),
            )]),
            Standard(IntakeAirTemperature) => Ok(vec![ObdValue::new(
                "Intake air temperature",
                ObdUnitType::Temperature(Temperature::from_celsius(
                    self.request_ecu(server, ff, 1)?[0] as f32 - 40.0,
                )),
            )]),
            Standard(MassAirFlow) => {
                let s = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Mass air flow sensor rate (Grames/sec)",
                    ObdUnitType::Raw(((s[0] as u32) << 8 | s[1] as u32) as f32 / 100.0),
                )])
            }
            Standard(ThrottlePosition) => {
                self.get_percentage_1_byte(server, ff, "Throttle position")
            }
            Standard(CommandedSecondaryAirStatus) => Ok(vec![ObdValue::new(
                "Commanded secondary air status",
                ObdUnitType::Encoded(ObdEnumValue::CommandedAirStatus(
                    CommandedSecondaryAirStatusByte::from(self.request_ecu(server, ff, 1)?[0]),
                )),
            )]),
            Standard(O2SensorsPresent2Banks) => Ok(vec![ObdValue::new(
                "Oxygen sensors present in 2 banks",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 1)?),
            )]),
            Standard(OxygenSensor1) => {
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
            Standard(OxygenSensor2) => {
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
            Standard(OxygenSensor3) => {
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
            Standard(OxygenSensor4) => {
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
            Standard(OxygenSensor5) => {
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
            Standard(OxygenSensor6) => {
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
            Standard(OxygenSensor7) => {
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
            Standard(OxygenSensor8) => {
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
            Standard(ObdStandard) => Ok(vec![ObdValue::new(
                "OBD Standard",
                ObdUnitType::Encoded(ObdEnumValue::ObdStandard(ObdStandardByte::from(
                    self.request_ecu(server, ff, 1)?[0],
                ))),
            )]),
            Standard(O2SensorsPresent4Banks) => Ok(vec![ObdValue::new(
                "Oxygen sensors present in 4 banks",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 1)?),
            )]),

            //Standard(AuxInputStatus) => {}
            Standard(RuntimeSinceStart) => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Runtime since engine start",
                    ObdUnitType::Time(Time::from_seconds(
                        ((r[0] as u32) << 8 | r[1] as u32) as f32,
                    )),
                )])
            }
            Standard(PidSupport2140) => Ok(vec![ObdValue::new(
                "PID support 21-40",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            Standard(MILRuntime) => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Distance travelled with MIL on",
                    ObdUnitType::Distance(Distance::from_kilometers(
                        ((r[0] as u32) << 8 | r[1] as u32) as f32,
                    )),
                )])
            }
            Standard(FuelRailPressure) => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Fuel rail pressure (Relative to manifold vacuum)",
                    ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                        ((r[0] as u32) << 8 | r[1] as u32) as f32 * 0.079,
                    )),
                )])
            }
            Standard(FuelRailGaugePressure) => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Fuel rail gauge pressure",
                    ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                        ((r[0] as u32) << 8 | r[1] as u32) as f32 * 10.0,
                    )),
                )])
            }
            Standard(OxygenSensor1LambdaVoltage) => {
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
            Standard(OxygenSensor2LambdaVoltage) => {
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
                            ((r[2] as u32) << 8 | r[3] as u32) as f32 * 0.000_1220703125,
                        ),
                    ),
                ])
            }
            Standard(OxygenSensor3LambdaVoltage) => {
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
            Standard(OxygenSensor4LambdaVoltage) => {
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
            Standard(OxygenSensor5LambdaVoltage) => {
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
            Standard(OxygenSensor6LambdaVoltage) => {
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
            Standard(OxygenSensor7LambdaVoltage) => {
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
            Standard(OxygenSensor8LambdaVoltage) => {
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
            Standard(CommandedEGR) => self.get_percentage_1_byte(server, ff, "Commanded EGR"),
            Standard(EGRError) => Ok(vec![ObdValue::new(
                "EGR Error",
                ObdUnitType::Percent(
                    self.request_ecu(server, ff, 1)?[0] as f32 * (100.0 / 128.0) - 100.0,
                ),
            )]),
            Standard(CommandedEvapPurge) => {
                self.get_percentage_1_byte(server, ff, "Commanded evaporative purge")
            }
            Standard(FuelTankLevelInput) => {
                self.get_percentage_1_byte(server, ff, "Fuel tank level input")
            }
            Standard(WarmupsSinceCodesCleared) => Ok(vec![ObdValue::new(
                "Warm-ups since codes cleared",
                ObdUnitType::Raw(self.request_ecu(server, ff, 1)?[0] as f32),
            )]),
            Standard(DistanceTraveledSinceCodesCleared) => {
                let r = self.request_ecu(server, ff, 2)?;
                Ok(vec![ObdValue::new(
                    "Distance traveled since codes cleared",
                    ObdUnitType::Distance(Distance::from_kilometers(
                        (((r[0] as u32) << 8) | r[1] as u32) as f32,
                    )),
                )])
            }
            Standard(EvapSystemVaporPressure) => {
                let r: Vec<i32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as i32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Evaporative System Vapor Pressure",
                    ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                        (r[0] << 8 | r[1]) as f32 / 4000.0,
                    )),
                )])
            }
            Standard(AbsBarometricPressure) => Ok(vec![ObdValue::new(
                "Absolute Barometric Pressure",
                ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                    self.request_ecu(server, ff, 1)?[0] as f32,
                )),
            )]),
            Standard(OxygenSensor1LambdaCurrent) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 4)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 1 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0 / 65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 1 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            Standard(OxygenSensor2LambdaCurrent) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 4)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 2 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0 / 65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 2 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            Standard(OxygenSensor3LambdaCurrent) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 4)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 3 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0 / 65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 3 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            Standard(OxygenSensor4LambdaCurrent) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 4)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 4 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0 / 65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 4 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            Standard(OxygenSensor5LambdaCurrent) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 4)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 5 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0 / 65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 5 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            Standard(OxygenSensor6LambdaCurrent) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 4)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 6 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0 / 65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 6 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            Standard(OxygenSensor7LambdaCurrent) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 4)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 7 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0 / 65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 7 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            Standard(OxygenSensor8LambdaCurrent) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 4)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Oxygen sensor 8 Lambda",
                        ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0 / 65536.0)),
                    ),
                    ObdValue::new(
                        "Oxygen sensor 8 Current",
                        ObdUnitType::Raw(((r[2] << 8 | r[3]) as f32 / 256.0) - 128.0),
                    ),
                ])
            }
            Standard(CatTempBank1Sensor1) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Catalyst Temperature bank 1, sensor 1",
                    ObdUnitType::Temperature(Temperature::from_celsius(
                        ((r[0] << 8 | r[1]) as f32 / 10.0) - 40.0,
                    )),
                )])
            }
            Standard(CatTempBank2Sensor1) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Catalyst Temperature bank 2, sensor 1",
                    ObdUnitType::Temperature(Temperature::from_celsius(
                        ((r[0] << 8 | r[1]) as f32 / 10.0) - 40.0,
                    )),
                )])
            }
            Standard(CatTempBank1Sensor2) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Catalyst Temperature bank 1, sensor 2",
                    ObdUnitType::Temperature(Temperature::from_celsius(
                        ((r[0] << 8 | r[1]) as f32 / 10.0) - 40.0,
                    )),
                )])
            }
            Standard(CatTempBank2Sensor2) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Catalyst Temperature bank 2, sensor 2",
                    ObdUnitType::Temperature(Temperature::from_celsius(
                        ((r[0] << 8 | r[1]) as f32 / 10.0) - 40.0,
                    )),
                )])
            }
            Standard(PidSupport4160) => Ok(vec![ObdValue::new(
                "PID support 41-60",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            Standard(MonitorStatusDriveCycle) => Ok(vec![ObdValue::new(
                "Monitor status this drive cycle",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            Standard(ControlModuleVoltage) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Control module voltage",
                    ObdUnitType::Volts((r[0] << 8 | r[1]) as f32 / 1000.0),
                )])
            }
            Standard(AbsLoadValue) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Absolute load value",
                    ObdUnitType::Percent((r[0] << 8 | r[1]) as f32 * (100.0 / 255.0)),
                )])
            }
            Standard(CommandedLambda) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Commanded Lambda",
                    ObdUnitType::Raw((r[0] << 8 | r[1]) as f32 * (2.0 / 65536.0)),
                )])
            }
            Standard(RelativeThrottlePosition) => {
                self.get_percentage_1_byte(server, ff, "Relative throttle position")
            }
            Standard(AmbientAirTemp) => Ok(vec![ObdValue::new(
                "Ambient air temperature",
                ObdUnitType::Temperature(Temperature::from_celsius(
                    self.request_ecu(server, ff, 1)?[0] as f32 - 40.0,
                )),
            )]),
            Standard(AbsoluteThrottlePositionB) => {
                self.get_percentage_1_byte(server, ff, "Absolute throttle position B")
            }
            Standard(AbsoluteThrottlePositionC) => {
                self.get_percentage_1_byte(server, ff, "Absolute throttle position C")
            }
            Standard(AbsoluteThrottlePositionD) => {
                self.get_percentage_1_byte(server, ff, "Absolute throttle position D")
            }
            Standard(AbsoluteThrottlePositionE) => {
                self.get_percentage_1_byte(server, ff, "Absolute throttle position E")
            }
            Standard(AbsoluteThrottlePositionF) => {
                self.get_percentage_1_byte(server, ff, "Absolute throttle position F")
            }
            Standard(CommandedThrottleActuator) => {
                self.get_percentage_1_byte(server, ff, "Commanded throttle actuator")
            }
            Standard(TimeRunSinceMILOn) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Time run with MIL on",
                    ObdUnitType::Time(Time::from_seconds((r[0] << 8 | r[1]) as f32 * 60.0)),
                )])
            }
            Standard(TimeSinceCodesCleared) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Time since trouble codes cleared",
                    ObdUnitType::Time(Time::from_seconds((r[0] << 8 | r[1]) as f32 * 60.0)),
                )])
            }
            Standard(MaximumLambdaVoltageCurrentPressure) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 4)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new("Maximum value for Lambda", ObdUnitType::Raw(r[0] as f32)),
                    ObdValue::new(
                        "Maximum value for oxygen sensor voltage",
                        ObdUnitType::Volts(r[1] as f32),
                    ),
                    ObdValue::new(
                        "Maximum value oxygen sensor current",
                        ObdUnitType::Raw(r[2] as f32),
                    ),
                    ObdValue::new(
                        "Maximum value for intake manifold absolute pressure",
                        ObdUnitType::Pressure(Pressure::from_kilo_pascal(r[3] as f32 * 10.0)),
                    ),
                ])
            }
            Standard(MaximumAirFlowRate) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 1)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Maximum value for air flow rate from mass air flow sensor",
                    ObdUnitType::Raw(r[0] as f32 * 10.0),
                )])
            }
            Standard(FuelType) => Ok(vec![ObdValue::new(
                "Fuel type",
                ObdUnitType::Encoded(ObdEnumValue::FuelType(FuelTypeCodingByte::from(
                    self.request_ecu(server, ff, 1)?[0],
                ))),
            )]),
            Standard(EthanolFuelPercentage) => {
                self.get_percentage_1_byte(server, ff, "Ethanol fuel")
            }
            Standard(AbsoluteEvapSystemVaporPressure) => {
                let r: Vec<i32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as i32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Absolute evaporative system vapor pressure",
                    ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                        (r[0] << 8 | r[1]) as f32 / 200.0,
                    )),
                )])
            }
            Standard(EvapSystemVaporPressure2) => {
                let r: Vec<i32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as i32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Evaporative system vapor pressure",
                    ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                        (r[0] << 8 | r[1]) as f32 / 1000.0,
                    )),
                )])
            }
            Standard(ShortTermSecondaryOxygenSensorTrimBank1and3) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Short term secondary oxygen sensor trim bank 1",
                        ObdUnitType::Percent(r[0] as f32 * (100.0 / 128.0) - 100.0),
                    ),
                    ObdValue::new(
                        "Short term secondary oxygen sensor trim bank 3",
                        ObdUnitType::Percent(r[1] as f32 * (100.0 / 128.0) - 100.0),
                    ),
                ])
            }
            Standard(LongTermSecondaryOxygenSensorTrimBank1and3) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Long term secondary oxygen sensor trim bank 1",
                        ObdUnitType::Percent(r[0] as f32 * (100.0 / 128.0) - 100.0),
                    ),
                    ObdValue::new(
                        "Long term secondary oxygen sensor trim bank 3",
                        ObdUnitType::Percent(r[1] as f32 * (100.0 / 128.0) - 100.0),
                    ),
                ])
            }
            Standard(ShortTermSecondaryOxygenSensorTrimBank2and4) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Short term secondary oxygen sensor trim bank 2",
                        ObdUnitType::Percent(r[0] as f32 * (100.0 / 128.0) - 100.0),
                    ),
                    ObdValue::new(
                        "Short term secondary oxygen sensor trim bank 4",
                        ObdUnitType::Percent(r[1] as f32 * (100.0 / 128.0) - 100.0),
                    ),
                ])
            }
            Standard(LongTermSecondaryOxygenSensorTrimBank2and4) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![
                    ObdValue::new(
                        "Long term secondary oxygen sensor trim bank 2",
                        ObdUnitType::Percent(r[0] as f32 * (100.0 / 128.0) - 100.0),
                    ),
                    ObdValue::new(
                        "Long term secondary oxygen sensor trim bank 4",
                        ObdUnitType::Percent(r[1] as f32 * (100.0 / 128.0) - 100.0),
                    ),
                ])
            }
            Standard(FuelRailAbsPressure) => {
                let r: Vec<u32> = self
                    .request_ecu(server, ff, 2)?
                    .iter()
                    .map(|x| *x as u32)
                    .collect();
                Ok(vec![ObdValue::new(
                    "Fuel rail absolute pressure",
                    ObdUnitType::Pressure(Pressure::from_kilo_pascal(
                        (r[0] << 8 | r[1]) as f32 * 10.0,
                    )),
                )])
            }
            Standard(RelativePedalPosition) => {
                self.get_percentage_1_byte(server, ff, "Relative accelerator pedal position")
            }
            Standard(HybridBatteryPackLife) => {
                self.get_percentage_1_byte(server, ff, "Hybrid battery pack remaining life")
            }
            /*
            Standard(EngineOilTemp) => {}
            Standard(FuelInjectionTiming) => {}
            Standard(EngineFuelRate) => {}
            */
            Standard(EmissionsStandard) => Ok(vec![ObdValue::new(
                "Emission requirments to which the vehicle is designed",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 1)?),
            )]),
            Standard(PidSupport6180) => Ok(vec![ObdValue::new(
                "PID support 61-80",
                ObdUnitType::ByteArray(self.request_ecu(server, ff, 4)?),
            )]),
            /*
            Standard(DriverDemandTorquePercent) => {}
            Standard(EngineTorquePercent) => {}
            Standard(EngineTorqueData) => {}
            Standard(AuxInputOutputSupport) => {}
            Standard(MassAirFlowSensor2) => {}
            Standard(EngineCoolantTemp2) => {}
            Standard(IntakeAirTemp2) => {}
            Standard(Unknown)(_) => {}
             */
            _ => Err(DiagError::NotImplemented(format!(
                "Parsing {:02X?}",
                self.0
            ))),
        }
    }
}
