//! Module for KWP2000 (Keyword protocol 2000 - ISO142330)
//!
//! This module is written to be 100% compliant with the following vehicle manufactures
//! which utilize KWP2000:
//! * Dodge
//! * Chrysler
//! * Jeep
//! * Mitsubishi
//! * Daimler (Mercedes-Benz and SMART)
//!
//! Other manufacturer's ECUs might also work, however they are untested.
//!
//! based on KWP2000 v2.2 (05/08/02)

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
    channel::{IsoTPChannel, IsoTPSettings},
    dtc::DTCFormatType,
    helpers, BaseServerPayload, BaseServerSettings, DiagError, DiagServerResult, ServerEvent,
    ServerEventHandler,
};

use self::start_diagnostic_session::SessionType;

pub mod clear_diagnostic_information;
pub mod ecu_reset;
pub mod read_data_by_identifier;
pub mod read_data_by_local_id;
pub mod read_dtc_by_status;
pub mod read_ecu_identification;
pub mod read_memory_by_address;
pub mod read_status_of_dtc;
pub mod security_access;
pub mod start_diagnostic_session;

/// KWP Command Service IDs.
///
/// Note. This does not cover both the 'Reserved' range (0x87-0xB9) and
/// 'System supplier specific' range (0xBA-0xBF)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum KWP2000Command {
    /// Start or change ECU diagnostic session mode. See [start_diagnostic_session]
    StartDiagnosticSession = 0x10,
    /// Reset the ECU. See [ecu_reset]
    ECUReset = 0x11,
    /// Clears diagnostic information stored on the ECU. See [clear_diagnostic_information]
    ClearDiagnosticInformation = 0x14,
    /// Reads snapshot data of DTCs stored on the ECU. See [read_status_of_dtc]
    ReadStatusOfDiagnosticTroubleCOdes = 0x17,
    /// Reads DTCs stored on the ECU. See [read_dtc_by_status]
    ReadDiagnosticTroubleCodesByStatus = 0x18,
    /// Reads ECU identification data. See [read_ecu_identification]
    ReadECUIdentification = 0x1A,
    /// Reads data from the ECU using a local identifier. See [read_data_by_local_id]
    ReadDataByLocalIdentifier = 0x21,
    /// Reads data from the ECU using a unique identifier. See [read_data_by_identifier]
    ReadDataByIdentifier = 0x22,
    /// Reads memory from the ECU by address. See [read_memory_by_address]
    ReadMemoryByAddress = 0x23,
    /// Security access functions. See [security_access]
    SecurityAccess = 0x27,
    ///
    DisableNormalMessageTransmission = 0x28,
    ///
    EnableNormalMessageTransmission = 0x29,
    ///
    DynamicallyDefineLocalIdentifier = 0x2C,
    ///
    WriteDataByIdentifier = 0x2E,
    ///
    InputOutputControlByLocalIdentifier = 0x30,
    ///
    StartRoutineByLocalIdentifier = 0x31,
    ///
    StopRoutineByLocalIdentifier = 0x32,
    ///
    RequestRoutineResultsByLocalIdentifier = 0x33,
    ///
    RequestDownload = 0x34,
    ///
    RequestUpload = 0x35,
    ///
    TransferData = 0x36,
    ///
    RequestTransferExit = 0x37,
    ///
    WriteDataByLocalIdentifier = 0x3B,
    ///
    WriteMemoryByAddress = 0x3D,
    ///
    TesterPresent = 0x3E,
    ///
    ControlDTCSettings = 0x85,
    ///
    ResponseOnEvent = 0x86,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
/// KWP Error definitions
pub enum KWP2000Error {
    /// ECU rejected the request for unknown reason
    GeneralReject,
    /// ECU Does not support the requested service
    ServiceNotSupported,
    /// ECU does not support arguments provided, or message format is incorrect
    SubFunctionNotSupportedInvalidFormat,
    /// ECU is too busy to perform the request
    BusyRepeatRequest,
    /// ECU prerequisite conditions are not met
    ConditionsNotCorrectRequestSequenceError,
    /// **Deprecated in v2.2 of KWP2000**. Requested results of a routine that is not completed.
    RoutineNotComplete,
    /// The request message contains data which is out of range
    RequestOutOfRange,
    /// Security access is denied
    SecurityAccessDenied,
    /// Invalid key provided to the ECU
    InvalidKey,
    /// Exceeded the number of incorrect security access attempts
    ExceedNumberOfAttempts,
    /// Time period for requesting a new seed not expired
    RequiredTimeDelayNotExpired,
    /// ECU fault prevents data download
    DownloadNotAccepted,
    /// ECU fault prevents data upload
    UploadNotAccepted,
    /// ECU fault has stopped the transfer of data
    TransferSuspended,
    /// The ECU has accepted the request, but cannot reply right now. If this error occurs,
    /// the [Kwp2000DiagnosticServer] will automatically stop sending tester present messages and
    /// will wait for the ECUs response. If after 2000ms, the ECU did not respond, then this error
    /// will get returned back to the function call.
    RequestCorrectlyReceivedResponsePending,
    /// Requested service is not supported in the current diagnostic session mode
    ServiceNotSupportedInActiveSession,
    /// Reserved for future ISO14230 use
    ReservedISO,
    /// Reserved for future use by DCX (Daimler)
    ReservedDCX,
    /// Data decompression failed
    DataDecompressionFailed,
    /// Data decryption failed
    DataDecryptionFailed,
    /// Sent by a gateway ECU. The requested ECU behind the gateway is not responding
    EcuNotResponding,
    /// Sent by a gateway ECU. The requested ECU address is unknown
    EcuAddressUnknown,
}

impl From<u8> for KWP2000Error {
    fn from(p: u8) -> Self {
        match p {
            0x10 => Self::GeneralReject,
            0x11 => Self::ServiceNotSupported,
            0x12 => Self::SubFunctionNotSupportedInvalidFormat,
            0x21 => Self::BusyRepeatRequest,
            0x22 => Self::ConditionsNotCorrectRequestSequenceError,
            0x23 => Self::RoutineNotComplete,
            0x31 => Self::RequestOutOfRange,
            0x33 => Self::SecurityAccessDenied,
            0x35 => Self::InvalidKey,
            0x36 => Self::ExceedNumberOfAttempts,
            0x37 => Self::RequiredTimeDelayNotExpired,
            0x40 => Self::DownloadNotAccepted,
            0x50 => Self::UploadNotAccepted,
            0x71 => Self::TransferSuspended,
            0x78 => Self::RequestCorrectlyReceivedResponsePending,
            0x80 => Self::ServiceNotSupportedInActiveSession,
            (0x90..=0x99) => Self::ReservedDCX,
            0x9A => Self::DataDecompressionFailed,
            0x9B => Self::DataDecryptionFailed,
            (0x9C..=0x9F) => Self::ReservedDCX,
            0xA0 => Self::EcuNotResponding,
            0xA1 => Self::EcuAddressUnknown,
            (0xA2..=0xF9) => Self::ReservedDCX,
            _ => Self::ReservedISO,
        }
    }
}

#[derive(Clone)]
/// Kwp2000 message payload
pub struct Kwp2000Cmd {
    bytes: Vec<u8>,
    response_required: bool,
}

impl std::fmt::Debug for Kwp2000Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Kwp2000Cmd")
            .field("Cmd", &self.get_kwp_sid())
            .field("Args", &self.get_payload())
            .field("response_required", &self.response_required)
            .finish()
    }
}

impl Kwp2000Cmd {
    /// Creates a new KWP2000 Payload
    pub fn new(sid: KWP2000Command, args: &[u8], need_response: bool) -> Self {
        let mut b: Vec<u8> = Vec::with_capacity(args.len() + 1);
        b.push(sid as u8);
        b.extend_from_slice(args);
        Self {
            bytes: b,
            response_required: need_response,
        }
    }

    /// Returns the KWP2000 Service ID of the command
    pub fn get_kwp_sid(&self) -> KWP2000Command {
        unsafe { transmute(self.bytes[0]) } // This unsafe operation will always succeed!
    }
}

impl BaseServerPayload for Kwp2000Cmd {
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

/// Base handler for KWP2000
#[derive(Debug, Copy, Clone)]
pub struct Kwp2000VoidHandler;

impl ServerEventHandler<SessionType, Kwp2000Cmd> for Kwp2000VoidHandler {
    #[inline(always)]
    fn on_event(&mut self, _e: ServerEvent<SessionType, Kwp2000Cmd>) {}
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
/// KWP2000 server options
pub struct Kwp2000ServerOptions {
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

impl BaseServerSettings for Kwp2000ServerOptions {
    fn get_write_timeout_ms(&self) -> u32 {
        self.write_timeout_ms
    }

    fn get_read_timeout_ms(&self) -> u32 {
        self.read_timeout_ms
    }
}

#[derive(Debug)]
/// Kwp2000 Diagnostic server
pub struct Kwp2000DiagnosticServer {
    server_running: Arc<AtomicBool>,
    settings: Kwp2000ServerOptions,
    tx: mpsc::Sender<Kwp2000Cmd>,
    rx: mpsc::Receiver<DiagServerResult<Vec<u8>>>,
    join_handler: JoinHandle<()>,
    repeat_count: u32,
    repeat_interval: std::time::Duration,
    dtc_format: Option<DTCFormatType>, // Used as a cache
}

impl Kwp2000DiagnosticServer {
    /// Creates a new KWP2000 over an ISO-TP connection with the ECU
    ///
    /// On startup, this server will configure the channel with the necessary settings provided in both
    /// settings and channel_cfg
    ///
    /// ## Parameters
    /// * settings - KWP2000 Server settings
    /// * channel - ISO-TP communication channel with the ECU
    /// * channel_cfg - The settings to use for the ISO-TP channel
    /// * event_handler - Handler for logging events happening within the server. If you don't want
    /// to create your own handler, use [Kwp2000VoidHandler]
    pub fn new_over_iso_tp<'a, C, E>(
        settings: Kwp2000ServerOptions,
        mut server_channel: C,
        channel_cfg: IsoTPSettings,
        mut event_handler: E,
    ) -> DiagServerResult<Self>
    where
        C: IsoTPChannel + 'static,
        E: ServerEventHandler<SessionType, Kwp2000Cmd> + 'static,
    {
        server_channel.set_iso_tp_cfg(channel_cfg)?;
        server_channel.set_ids(settings.send_id, settings.recv_id)?;
        server_channel.open()?;

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_t = is_running.clone();

        let (tx_cmd, rx_cmd) = mpsc::channel::<Kwp2000Cmd>();
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
                    if cmd.get_kwp_sid() == KWP2000Command::StartDiagnosticSession {
                        // Session change! Handle this differently
                        match helpers::perform_cmd(
                            settings.send_id,
                            &cmd,
                            &settings,
                            &mut server_channel,
                            0x78,
                            0x21,
                        ) {
                            // 0x78 - Response correctly received, response pending
                            Ok(res) => {
                                // Set server session type
                                if cmd.bytes[1] == u8::from(SessionType::Passive)
                                    || cmd.bytes[1] == u8::from(SessionType::Normal)
                                {
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
                            0x78,
                            0x21,
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
                    let arg = if settings.tester_present_require_response {
                        0x01
                    } else {
                        0x02
                    };

                    let cmd = Kwp2000Cmd::new(
                        KWP2000Command::TesterPresent,
                        &[arg],
                        settings.tester_present_require_response,
                    );
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

    /// Returns true if the internal KWP2000 Server is running
    pub fn is_server_running(&self) -> bool {
        self.server_running.load(Ordering::Relaxed)
    }

    /// Returns the current settings used by the KWP2000 Server
    pub fn get_settings(&self) -> Kwp2000ServerOptions {
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
        sid: KWP2000Command,
        args: &[u8],
    ) -> DiagServerResult<Vec<u8>> {
        let cmd = Kwp2000Cmd::new(sid, args, true);

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
    pub fn execute_command(&mut self, sid: KWP2000Command, args: &[u8]) -> DiagServerResult<()> {
        let cmd = Kwp2000Cmd::new(sid, args, false);
        self.exec_command(cmd).map(|_| ())
    }

    /// Internal command for sending KWP2000 payload to the ECU
    fn exec_command(&mut self, cmd: Kwp2000Cmd) -> DiagServerResult<Vec<u8>> {
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

/// Returns the KWP2000 error from a given error code
pub fn get_description_of_ecu_error(error: u8) -> KWP2000Error {
    error.into()
}
