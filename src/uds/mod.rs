//! Module for UDS (Unified diagnostic services - ISO14229)
//!
//! Theoretically, this module should be compliant with any ECU which implements
//! UDS (Typically any ECU produced after 2006 supports this)

use std::{
    intrinsics::transmute,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::JoinHandle,
    time::Instant,
};

use crate::{
    channel::IsoTPChannel, channel::IsoTPSettings, dtc::DTCFormatType, helpers, BaseServerPayload,
    BaseServerSettings, DiagError, DiagServerResult, ServerEvent, ServerEventHandler,
};

use self::diagnostic_session_control::UDSSessionType;

pub mod diagnostic_session_control;
pub mod ecu_reset;
pub mod read_dtc_information;
pub mod security_access;

/// UDS Command Service IDs
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
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
    /// Reading and querying diagnostic trouble codes
    /// stored on the ECU. See [read_dtc_information]
    ReadDTCInformation = 0x19,
    InputOutputControlByIdentifier = 0x2F,
    RoutineControl = 0x31,
    RequestDownload = 0x34,
    RequestUpload = 0x35,
    TransferData = 0x36,
    RequestTransferExit = 0x37,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
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
#[repr(C)]
/// UDS server options
pub struct UdsServerOptions {
    /// ECU Send ID
    pub send_id: u32,
    /// ECU Receive ID
    pub recv_id: u32,
    /// Read timeout in ms
    pub read_timeout_ms: u32,
    /// Write timeout in ms
    pub write_timeout_ms: u32,
    /// Optional global address to send tester-present messages to
    /// Set to 0 if not in use
    pub global_tp_id: u32,
    /// Tester present minimum send interval in ms
    pub tester_present_interval_ms: u32,
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

#[derive(Clone)]
/// UDS message payload
pub struct UdsCmd {
    bytes: Vec<u8>,
    response_required: bool,
}

impl std::fmt::Debug for UdsCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UdsCmd")
            .field("Cmd", &self.get_uds_sid())
            .field("Args", &self.get_payload())
            .field("response_required", &self.response_required)
            .finish()
    }
}

impl UdsCmd {
    /// Creates a new UDS Payload
    pub fn new(sid: UDSCommand, args: &[u8], need_response: bool) -> Self {
        let mut b: Vec<u8> = Vec::with_capacity(args.len() + 1);
        b.push(sid as u8);
        b.extend_from_slice(args);
        Self {
            bytes: b,
            response_required: need_response,
        }
    }

    /// Returns the UDS Service ID of the command
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

/// Base handler for UDS
#[derive(Debug, Copy, Clone)]
pub struct UdsVoidHandler;

impl ServerEventHandler<UDSSessionType, UdsCmd> for UdsVoidHandler {
    #[inline(always)]
    fn on_event(&mut self, _e: ServerEvent<UDSSessionType, UdsCmd>) {}
}

#[derive(Debug)]
/// UDS Diagnostic server
pub struct UdsDiagnosticServer {
    server_running: Arc<AtomicBool>,
    settings: UdsServerOptions,
    tx: mpsc::Sender<UdsCmd>,
    rx: mpsc::Receiver<DiagServerResult<Vec<u8>>>,
    join_handler: JoinHandle<()>,
    repeat_count: u32,
    repeat_interval: std::time::Duration,
    dtc_format: Option<DTCFormatType>, // Used as a cache
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
    /// * event_handler - Handler for logging events happening within the server. If you don't want
    /// to create your own handler, use [UdsVoidHandler]
    pub fn new_over_iso_tp<C, E>(
        settings: UdsServerOptions,
        mut server_channel: C,
        channel_cfg: IsoTPSettings,
        mut event_handler: E,
    ) -> DiagServerResult<Self>
    where
        C: IsoTPChannel + 'static,
        E: ServerEventHandler<UDSSessionType, UdsCmd> + 'static,
    {
        server_channel.set_iso_tp_cfg(channel_cfg)?;
        server_channel.set_ids(settings.send_id, settings.recv_id)?;
        server_channel.open()?;

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_t = is_running.clone();

        let (tx_cmd, rx_cmd) = mpsc::channel::<UdsCmd>();
        let (tx_res, rx_res) = mpsc::channel::<DiagServerResult<Vec<u8>>>();

        let handle = std::thread::spawn(move || {
            let mut send_tester_present = false;
            let mut last_tester_present_time: Instant = Instant::now();

            event_handler.on_event(ServerEvent::ServerStart);

            loop {
                if !is_running_t.load(Ordering::Relaxed) {
                    break;
                }

                if let Ok(cmd) = rx_cmd.try_recv() {
                    event_handler.on_event(ServerEvent::IncomingEvent(&cmd));
                    // We have an incoming command
                    if cmd.get_uds_sid() == UDSCommand::DiagnosticSessionControl {
                        // Session change! Handle this differently
                        match helpers::perform_cmd(
                            settings.send_id,
                            &cmd,
                            &settings,
                            &mut server_channel,
                            0x78,
                            0x21
                        ) {
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
                                if tx_res.send(Ok(res)).is_err() {
                                    // Terminate! Something has gone wrong and data can no longer be sent to client
                                    is_running_t.store(false, Ordering::Relaxed);
                                    event_handler.on_event(ServerEvent::CriticalError {
                                        desc: "Channel Tx SendError occurred".into(),
                                    })
                                }
                            }
                            Err(e) => {
                                if tx_res.send(Err(e)).is_err() {
                                    // Terminate! Something has gone wrong and data can no longer be sent to client
                                    is_running_t.store(false, Ordering::Relaxed);
                                    event_handler.on_event(ServerEvent::CriticalError {
                                        desc: "Channel Tx SendError occurred".into(),
                                    })
                                }
                            }
                        }
                    } else {
                        // Generic command just perform it
                        let res = helpers::perform_cmd(
                            settings.send_id,
                            &cmd,
                            &settings,
                            &mut server_channel,
                            0x78, // UDSError::RequestCorrectlyReceivedResponsePending
                            0x21
                        );
                        //event_handler.on_event(&res);
                        if tx_res.send(res).is_err() {
                            // Terminate! Something has gone wrong and data can no longer be sent to client
                            is_running_t.store(false, Ordering::Relaxed);
                            event_handler.on_event(ServerEvent::CriticalError {
                                desc: "Channel Tx SendError occurred".into(),
                            })
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
                    let addr = match settings.global_tp_id {
                        0 => settings.send_id,
                        x => x,
                    };

                    if let Err(e) =
                        helpers::perform_cmd(addr, &cmd, &settings, &mut server_channel, 0x78, 0x21)
                    {
                        event_handler.on_event(ServerEvent::TesterPresentError(e))
                    }
                    last_tester_present_time = Instant::now();
                }

                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            // Goodbye server
            event_handler.on_event(ServerEvent::ServerExit);
            if let Err(e) = server_channel.close() {
                event_handler.on_event(ServerEvent::InterfaceCloseOnExitError(e))
            }
        });

        Ok(Self {
            server_running: is_running,
            tx: tx_cmd,
            rx: rx_res,
            settings,
            join_handler: handle,
            repeat_count: 3,
            repeat_interval: std::time::Duration::from_millis(1000),
            dtc_format: None,
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

        if self.repeat_count == 0 {
            self.exec_command(cmd)
        } else {
            let mut last_err: Option<DiagError> = None;
            for _ in 0..self.repeat_count {
                let start = Instant::now();
                match self.exec_command(cmd.clone()) {
                    Ok(resp) => return Ok(resp),
                    Err(e) => {
                        if let DiagError::ECUError(_) = e {
                            return Err(e); // ECU Error. Sending again won't help.
                        }
                        last_err = Some(e); // Other error. Sleep and then try again
                        if let Some(sleep_time) = self.repeat_interval.checked_sub(start.elapsed())
                        {
                            std::thread::sleep(sleep_time)
                        }
                    }
                }
            }
            Err(last_err.unwrap())
        }
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
            Err(_) => Err(DiagError::ServerNotRunning), // Server must have crashed!
        }
    }

    /// Sets the command retry counter
    pub fn set_repeat_count(&mut self, count: u32) {
        self.repeat_count = count
    }

    /// Sets the command retry interval
    pub fn set_repeat_interval_count(&mut self, interval_ms: u32) {
        self.repeat_interval = std::time::Duration::from_millis(interval_ms as u64)
    }
}

/// Returns the [UDSError] from a matching input byte.
/// The error byte provided MUST come from [DiagError::ECUError]
///
/// ## Example:
/// ```
/// #extern crate ecu_diagnostics;
/// #use ecu_diagnostics::{DiagError, uds};
///
/// let result = DiagError::ECUError(0x10);
///
/// if let DiagError::ECUError(x) = result {
///     let error_name = uds::get_description_of_ecu_error(x);
///     println!("ECU Rejected request: {:?}", error_name);
///     assert_eq!(error_name, uds::UDSError::GeneralReject);
/// } else {
///     println!("Non-ECU error performing request: {:?}", result);
/// }
///
/// ```
pub fn get_description_of_ecu_error(error: u8) -> UDSError {
    error.into()
}

unsafe impl Sync for UdsDiagnosticServer {}
unsafe impl Send for UdsDiagnosticServer {}

#[cfg(test)]
pub mod uds_test {
    use std::collections::HashMap;

    use crate::channel::{BaseChannel, ChannelError, ChannelResult};

    use super::*;

    #[derive(Debug, Clone)]
    pub struct FakeIsoTpChannel {
        triggers: HashMap<(u8, Option<u8>), Vec<u8>>,
        resp_queue: Vec<Vec<u8>>,
    }

    impl FakeIsoTpChannel {
        pub fn new() -> Self {
            Self {
                triggers: HashMap::new(),
                resp_queue: Vec::new(),
            }
        }

        pub fn add_sid_respose(&mut self, sid: u8, pid: Option<u8>, resp: &[u8]) {
            self.triggers.insert((sid, pid), resp.to_vec());
        }
    }

    impl IsoTPChannel for FakeIsoTpChannel {
        fn set_iso_tp_cfg(&mut self, _cfg: IsoTPSettings) -> crate::channel::ChannelResult<()> {
            Ok(())
        }
    }

    impl BaseChannel for FakeIsoTpChannel {
        fn open(&mut self) -> crate::channel::ChannelResult<()> {
            Ok(())
        }

        fn close(&mut self) -> crate::channel::ChannelResult<()> {
            Ok(())
        }

        fn set_ids(&mut self, _send: u32, _recv: u32) -> crate::channel::ChannelResult<()> {
            Ok(())
        }

        fn read_bytes(&mut self, _timeout_ms: u32) -> crate::channel::ChannelResult<Vec<u8>> {
            if self.resp_queue.len() > 0 {
                let ret = self.resp_queue[0].clone();
                self.resp_queue.drain(0..1);
                return Ok(ret);
            }
            Err(ChannelError::ReadTimeout)
        }

        fn write_bytes(
            &mut self,
            _addr: u32,
            buffer: &[u8],
            _timeout_ms: u32,
        ) -> crate::channel::ChannelResult<()> {
            // Pretend we are in the ECU here
            if buffer.len() == 1 {
                if let Some(((sid, _), buf)) = self.triggers.get_key_value(&(buffer[0], None)) {
                    // Check for function
                    // Function found
                    let mut res = vec![sid + 0x40];
                    res.extend_from_slice(buf);
                    self.resp_queue.push(res)
                } else {
                    // Function not in trigger!? Respond with unsupported
                    self.resp_queue.push(vec![0x7F, buffer[0], 0x11]) // 0x11 - Service not supported
                }
            }

            if let Some(((sid, pid), buf)) =
                self.triggers.get_key_value(&(buffer[0], Some(buffer[1])))
            {
                // Check for function
                // Function found
                let mut res = vec![sid + 0x40, buffer[1]];
                res.extend_from_slice(buf);
                self.resp_queue.push(res)
            } else {
                // Function not in trigger!? Respond with unsupported
                self.resp_queue.push(vec![0x7F, buffer[0], 0x11]) // 0x11 - Service not supported
            }
            Ok(())
        }

        fn clear_rx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
            Ok(self.resp_queue.clear())
        }

        fn clear_tx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct TestUdsServer {
        pub uds: super::UdsDiagnosticServer,
    }

    impl TestUdsServer {
        pub fn new(fake_channel_data: FakeIsoTpChannel) -> Self {
            let server = UdsDiagnosticServer::new_over_iso_tp(
                UdsServerOptions {
                    send_id: 0x01,
                    recv_id: 0x02,
                    read_timeout_ms: 1000,
                    write_timeout_ms: 1000,
                    global_tp_id: 0,
                    tester_present_interval_ms: 2000,
                    tester_present_require_response: false,
                },
                fake_channel_data,
                IsoTPSettings {
                    block_size: 20,
                    st_min: 8,
                    extended_addressing: false,
                    pad_frame: true,
                    can_speed: 500000,
                    can_use_ext_addr: false,
                },
                UdsVoidHandler,
            )
            .unwrap();
            Self { uds: server }
        }
    }
}
