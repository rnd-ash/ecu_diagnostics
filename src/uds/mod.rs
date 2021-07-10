//! Module for UDS (Unified diagnostic services - ISO14229)
//!
//! Theoretically, this module should be compliant with any ECU which implements
// UDS (Typically any ECU produced after 2006 supports this) 

use std::{
    intrinsics::transmute,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Instant,
};

use crate::{
    helpers, BaseChannel, BaseServerPayload, BaseServerSettings, DiagError, DiagServerLogger,
    DiagServerResult, IsoTPChannel, IsoTPSettings,
};

use self::diagnostic_session_control::UDSSessionType;

pub mod diagnostic_session_control;
pub mod ecu_reset;
pub mod security_access;

#[cfg(test)]
mod test;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// UDS Command Service IDs
pub enum UDSCommand {
    /// Diagnostic session control. See [diagnostic_session_control]
    DiagnosticSessionControl = 0x10,
    /// ECU Reset. See [ecu_reset]
    ECUReset = 0x11,
    /// Security access. See [security_access]
    SecurityAccess = 0x27,
    /// Controls communication functionality of the ECU
    CommunicationControl = 0x28,
    /// Tester present command.
    TesterPresent = 0x3E,
    AccessTimingParameters = 0x83,
    SecuredDataTransmission = 0x84,
    ControlDTCSettings = 0x85,
    ResponseOnEvent = 0x86,
    LinkControl = 0x87,
    ReadDataByIdentifier = 0x22,
    ReadMemoryByAddress = 0x23,
    ReadScalingDataByIdentifier = 0x24,
    ReadDataByPeriodicIdentifier = 0x2A,
    DynamicallyDefineDataIdentifier = 0x2C,
    WriteDataByIdentifier = 0x2E,
    WriteMemoryByAddress = 0x3D,
    ClearDiagnosticInformation = 0x14,
    ReadDTCInformation = 0x19,
    InputOutputControlByIdentifier = 0x2F,
    RoutineControl = 0x31,
    RequestDownload = 0x34,
    RequestUpload = 0x35,
    TransferData = 0x36,
    RequestTransferExit = 0x37,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// UDS Error definitions
pub enum UDSError {
    /// ECU rejected the request (No specific error)
    GeneralReject,
    /// Service is not supported by the ECU
    ServiceNotSupported,
    /// Sub function is not supported by the ECU
    SubFunctionNotSupported,
    /// Request message was an invalid length, or the format of the
    /// request was incorrect
    IncorrectMessageLengthOrInvalidFormat,
    /// The response message is too long for the transport protocol
    ResponseTooLong,
    /// The ECU is too busy to perform this request. Therefore, the request
    /// Should be sent again if this error occurs
    BusyRepeatRequest,
    /// The requested action could not be preformed due to the prerequisite conditions
    /// not being correct
    ConditionsNotCorrect,
    /// The ECU cannot perform the request as the request has been sent in the incorrect order.
    /// For example, if [security_access::send_key] is used before [security_access::request_seed],
    /// then the ECU will respond with this error.
    RequestSequenceError,
    /// The ECU cannot perform the request as it has timed out trying to communicate with another 
    /// component within the vehicle.
    NoResponseFromSubnetComponent,
    /// The ECU cannot perform the requested action as there is currently a DTC 
    /// or failure of a component that is preventing the execution of the request.
    FailurePreventsExecutionOfRequestedAction,
    /// The request message contains data outside of a valid range
    RequestOutOfRange,
    /// The request could not be completed due to security access being denied.
    SecurityAccessDenied,
    /// The key sent from [security_access::send_key] was invalid
    InvalidKey,
    /// The client has tried to obtain security access to the ECU too many times with
    /// incorrect keys
    ExceedNumberOfAttempts,
    /// The client has tried to request seed_key's too quickly, before the ECU timeout's period
    /// has expired
    RequiredTimeDelayNotExpired,
    /// The ECU cannot accept the requested upload/download request due to a fault condition
    UploadDownloadNotAccepted,
    /// The ECU has halted data transfer due to a fault condition
    TransferDataSuspended,
    /// The ECU has encountered an error during reprogramming (erasing / flashing)
    GeneralProgrammingFailure,
    /// The ECU has detected the reprogramming error as the blockSequenceCounter is incorrect.
    WrongBlockSequenceCounter,
    /// The ECU has accepted the request, but cannot reply right now. If this error occurs,
    /// the [UdsDiagnosticServer] will automatically stop sending tester present messages and
    /// will wait for the ECUs response. If after 2000ms, the ECU did not respond, then this error
    /// will get returned back to the function call.
    RequestCorrectlyReceivedResponsePending,
    /// The sub function is not supported in the current diagnostic session mode
    SubFunctionNotSupportedInActiveSession,
    /// The service is not supported in the current diagnostic session mode
    ServiceNotSupportedInActiveSession,
    /// Engine RPM is too high
    RpmTooHigh,
    /// Engine RPM is too low
    RpmTooLow,
    /// Engine is running
    EngineIsRunning,
    /// Engine is not running
    EngineIsNotRunning,
    /// Engine has not been running for long enough
    EngineRunTimeTooLow,
    /// Engine temperature (coolant) is too high
    TemperatureTooHigh,
    /// Engine temperature (coolant) is too low
    TemperatureTooLow,
    /// Vehicle speed is too high
    VehicleSpeedTooHigh,
    /// Vehicle speed is too low
    VehicleSpeedTooLow,
    /// Throttle or pedal value is too high
    ThrottleTooHigh,
    /// Throttle or pedal value is too low
    ThrottleTooLow,
    /// Transmission is not in neutral
    TransmissionRangeNotInNeutral,
    /// Transmission is not in gear
    TransmissionRangeNotInGear,
    /// Brake is not applied
    BrakeSwitchNotClosed,
    /// Shifter lever is not in park
    ShifterLeverNotInPark,
    /// Automatic/CVT transmission torque convert is locked
    TorqueConverterClutchLocked,
    /// Voltage is too high
    VoltageTooHigh,
    /// Voltage is too low
    VoltageTooLow,
    /// (0x94-0xFE) This range is reserved for future definition.
    ReserverdForSpecificConditionsNotCorrect,
    /// (0x38-0x4F) This range of values is reserved for ISO-15765 data link security
    ReservedByExtendedDataLinkSecurityDocumentation,
    /// Other reserved error code
    IsoSAEReserved(u8),
}

impl From<u8> for UDSError {
    fn from(p: u8) -> Self {
        match p {
            0x10 => Self::GeneralReject,
            0x11 => Self::ServiceNotSupported,
            0x12 => Self::SubFunctionNotSupported,
            0x13 => Self::IncorrectMessageLengthOrInvalidFormat,
            0x14 => Self::ResponseTooLong,
            0x21 => Self::BusyRepeatRequest,
            0x22 => Self::ConditionsNotCorrect,
            0x24 => Self::RequestSequenceError,
            0x25 => Self::NoResponseFromSubnetComponent,
            0x26 => Self::FailurePreventsExecutionOfRequestedAction,
            0x31 => Self::RequestOutOfRange,
            0x33 => Self::SecurityAccessDenied,
            0x35 => Self::InvalidKey,
            0x36 => Self::ExceedNumberOfAttempts,
            0x37 => Self::RequiredTimeDelayNotExpired,
            0x70 => Self::UploadDownloadNotAccepted,
            0x71 => Self::TransferDataSuspended,
            0x72 => Self::GeneralProgrammingFailure,
            0x73 => Self::WrongBlockSequenceCounter,
            0x78 => Self::RequestCorrectlyReceivedResponsePending,
            0x7E => Self::SubFunctionNotSupportedInActiveSession,
            0x7F => Self::ServiceNotSupportedInActiveSession,
            0x81 => Self::RpmTooHigh,
            0x82 => Self::RpmTooLow,
            0x83 => Self::EngineIsRunning,
            0x84 => Self::EngineIsNotRunning,
            0x85 => Self::EngineRunTimeTooLow,
            0x86 => Self::TemperatureTooHigh,
            0x87 => Self::TemperatureTooLow,
            0x88 => Self::VehicleSpeedTooHigh,
            0x89 => Self::VehicleSpeedTooLow,
            0x8A => Self::ThrottleTooHigh,
            0x8B => Self::ThrottleTooLow,
            0x8C => Self::TransmissionRangeNotInNeutral,
            0x8D => Self::TransmissionRangeNotInGear,
            0x8F => Self::BrakeSwitchNotClosed,
            0x90 => Self::ShifterLeverNotInPark,
            0x91 => Self::TorqueConverterClutchLocked,
            0x92 => Self::VoltageTooHigh,
            0x93 => Self::VoltageTooLow,
            (0x94..=0xFE) => Self::ReserverdForSpecificConditionsNotCorrect,
            (0x38..=0x4F) => Self::ReservedByExtendedDataLinkSecurityDocumentation,
            x => Self::IsoSAEReserved(x),
        }
    }
}

#[derive(Debug, Copy, Clone)]
/// UDS server options
pub struct UdsServerOptions {
    /// Baud rate (Connection speed)
    pub baud: u32,
    /// ECU Send ID
    pub send_id: u32,
    /// ECU Receive ID
    pub recv_id: u32,
    /// Read timeout in ms
    pub read_timeout_ms: u32,
    /// Write timeout in ms
    pub write_timeout_ms: u32,
    /// Optional global address to send tester-present messages to
    pub global_tp_id: Option<u32>,
    /// Tester present minimum send interval in ms
    pub tester_present_interval_ms: u32,
    /// Server refresh interval (For writing/reading). A sensible value is 10ms
    pub server_refresh_interval_ms: u32,
    /// Configures if the diagnostic server will poll for a response from tester present.
    pub tester_present_require_response: bool,
}

impl BaseServerSettings for UdsServerOptions {
    fn get_write_timeout_ms(&self) -> u32 {
        self.write_timeout_ms
    }

    fn get_read_timeout_ms(&self) -> u32 {
        self.read_timeout_ms
    }
}

#[derive(Debug, Clone)]
/// UDS message payload
pub struct UdsCmd {
    bytes: Vec<u8>,
    response_required: bool,
}

impl UdsCmd {
    pub fn new(sid: UDSCommand, args: &[u8], need_response: bool) -> Self {
        let mut b: Vec<u8> = Vec::with_capacity(args.len() + 1);
        b.push(sid as u8);
        b.extend_from_slice(args);
        Self {
            bytes: b,
            response_required: need_response,
        }
    }
    pub fn get_uds_sid(&self) -> UDSCommand {
        unsafe { transmute(self.bytes[0]) } // This unsafe operation will always succeed!
    }
}

impl BaseServerPayload for UdsCmd {
    fn get_payload(&self) -> &[u8] {
        &self.bytes[1..]
    }

    fn get_sid_byte(&self) -> u8 {
        self.bytes[0]
    }

    fn to_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn requires_response(&self) -> bool {
        self.response_required
    }
}

#[derive(Debug)]
/// UDS Diagnostic server
pub struct UdsDiagnosticServer {
    server_running: Arc<AtomicBool>,
    settings: UdsServerOptions,
    tx: mpsc::Sender<UdsCmd>,
    rx: mpsc::Receiver<DiagServerResult<Vec<u8>>>,
}

impl UdsDiagnosticServer {
    /// Creates a new UDS over an ISO-TP connection with the ECU
    ///
    /// On startup, this server will configure the channel with the necessary settings provided in both
    /// settings and channel_cfg
    ///
    /// ## Parameters
    /// * settings - UDS Server settings
    /// * channel - ISO-TP communication channel with the ECU
    /// * channel_cfg - The settings to use for the ISO-TP channel
    /// * event_handler - Optional handler for logging events happening within the server
    pub fn new_over_iso_tp(
        settings: UdsServerOptions,
        channel: Box<dyn IsoTPChannel>,
        channel_cfg: IsoTPSettings,
        event_handler: Option<Box<dyn DiagServerLogger<UDSSessionType, UdsCmd>>>,
    ) -> DiagServerResult<Self> {
        let mut server_channel = channel.clone();

        server_channel.set_baud(settings.baud)?;
        server_channel.set_ids(settings.send_id, settings.recv_id, settings.global_tp_id)?;
        server_channel.configure_iso_tp(channel_cfg)?;

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_t = is_running.clone();

        let (tx_cmd, rx_cmd) = mpsc::channel::<UdsCmd>();
        let (tx_res, rx_res) = mpsc::channel::<DiagServerResult<Vec<u8>>>();

        let server_opts = settings.clone();

        std::thread::spawn(move || {
            let mut send_tester_present = false;
            let mut last_tester_present_time: Instant = Instant::now();

            let mut base_channel = server_channel.clone_base();

            if let Some(h) = &event_handler {
                h.on_server_start()
            }

            loop {
                if is_running_t.load(Ordering::Relaxed) == false {
                    if let Some(h) = &event_handler {
                        h.on_server_exit()
                    }
                    break;
                }

                if let Ok(cmd) = rx_cmd.try_recv() {
                    // We have an incoming command
                    if cmd.get_uds_sid() == UDSCommand::DiagnosticSessionControl {
                        // Session change! Handle this differently
                        match helpers::perform_cmd(&cmd, &settings, &mut base_channel, 0x78) {
                            // 0x78 - Response correctly received, response pending
                            Ok(res) => {
                                // Set server session type
                                if cmd.bytes[1] == UDSSessionType::Default.into() {
                                    // Default session, disable tester present
                                    send_tester_present = false;
                                } else {
                                    // Enable tester present and refresh the delay
                                    send_tester_present = true;
                                    last_tester_present_time = Instant::now();
                                }
                                // Send response to client
                                if let Err(_) = tx_res.send(Ok(res)) {
                                    // Terminate! Something has gone wrong and data can no longer be sent to client
                                    is_running_t.store(false, Ordering::Relaxed);
                                    if let Some(h) = &event_handler {
                                        h.on_critical_error("Channel Tx SendError occurred");
                                    }
                                }
                            }
                            Err(e) => {
                                if let Err(_) = tx_res.send(Err(e)) {
                                    // Terminate! Something has gone wrong and data can no longer be sent to client
                                    is_running_t.store(false, Ordering::Relaxed);
                                    if let Some(h) = &event_handler {
                                        h.on_critical_error("Channel Tx SendError occurred");
                                    }
                                }
                            }
                        }
                    } else {
                        // Generic command just perform it
                        if let Err(_) = tx_res.send(helpers::perform_cmd(
                            &cmd,
                            &settings,
                            &mut base_channel,
                            0x78,
                        )) {
                            // Terminate! Something has gone wrong and data can no longer be sent to client
                            is_running_t.store(false, Ordering::Relaxed);
                            if let Some(h) = &event_handler {
                                h.on_critical_error("Channel Tx SendError occurred");
                            }
                        }
                    }
                }

                // Deal with tester present
                if send_tester_present
                    && last_tester_present_time.elapsed().as_millis() as u32
                        >= settings.tester_present_interval_ms
                {
                    // Send tester present message
                    let cmd = UdsCmd::new(UDSCommand::TesterPresent, &[0x00], true);
                    if let Err(e) = helpers::perform_cmd(&cmd, &settings, &mut base_channel, 0x78) {
                        if let Some(h) = &event_handler {
                            h.on_tester_present_error(e)
                        }
                    }
                    last_tester_present_time = Instant::now();
                }

                std::thread::sleep(std::time::Duration::from_millis(
                    server_opts.server_refresh_interval_ms as u64,
                ));
            }
        });

        Ok(Self {
            server_running: is_running,
            tx: tx_cmd,
            rx: rx_res,
            settings,
        })
    }

    /// Returns true if the internal UDS Server is running
    pub fn is_server_running(&self) -> bool {
        self.server_running.load(Ordering::Relaxed)
    }

    /// Returns the current settings used by the UDS Server
    pub fn get_settings(&self) -> UdsServerOptions {
        self.settings
    }

    /// Send a command to the ECU, and receive its response
    ///
    /// ## Parameters
    /// * sid - The Service ID of the command
    /// * args - The arguments for the service
    ///
    /// ## Returns
    /// If the function is successful, and the ECU responds with an OK response (Containing data),
    /// then the full ECU response is returned. The response will begin with the sid + 0x40
    pub fn execute_command_with_response(
        &mut self,
        sid: UDSCommand,
        args: &[u8],
    ) -> DiagServerResult<Vec<u8>> {
        let cmd = UdsCmd::new(sid, args, true);
        self.exec_command(cmd)
    }

    /// Send a command to the ECU, but don't receive a response
    ///
    /// ## Parameters
    /// * sid - The Service ID of the command
    /// * args - The arguments for the service
    pub fn execute_command(&mut self, sid: UDSCommand, args: &[u8]) -> DiagServerResult<()> {
        let cmd = UdsCmd::new(sid, args, false);
        self.exec_command(cmd).map(|_| ())
    }

    /// Internal command for sending UDS payload to the ECU
    fn exec_command(&mut self, cmd: UdsCmd) -> DiagServerResult<Vec<u8>> {
        match self.tx.send(cmd) {
            Ok(_) => self.rx.recv().unwrap_or(Err(DiagError::ServerNotRunning)),
            Err(_) => return Err(DiagError::ServerNotRunning), // Server must have crashed!
        }
    }
}

unsafe impl Sync for UdsDiagnosticServer {}
unsafe impl Send for UdsDiagnosticServer {}
