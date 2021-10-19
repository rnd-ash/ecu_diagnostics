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
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::JoinHandle,
    time::Instant,
};

use crate::{BaseServerPayload, BaseServerSettings, DiagError, DiagServerResult, DiagnosticServer, ServerEvent, ServerEventHandler, channel::{IsoTPChannel, IsoTPSettings}, dtc::DTCFormatType, helpers};

mod clear_diagnostic_information;
mod ecu_reset;
mod read_data_by_identifier;
mod read_data_by_local_id;
mod read_dtc_by_status;
mod read_ecu_identification;
mod read_memory_by_address;
mod read_status_of_dtc;
mod security_access;
mod start_diagnostic_session;
mod routine;
mod message_transmission;


pub use clear_diagnostic_information::*;
pub use ecu_reset::*;
pub use read_data_by_identifier::*;
pub use read_data_by_local_id::*;
pub use read_dtc_by_status::*;
pub use read_ecu_identification::*;
pub use read_memory_by_address::*;
pub use read_status_of_dtc::*;
pub use security_access::*;
pub use start_diagnostic_session::*;
pub use routine::*;
pub use message_transmission::*;

/// KWP Command Service IDs.
///
/// Note. This does not cover both the 'Reserved' range (0x87-0xB9) and
/// 'System supplier specific' range (0xBA-0xBF)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum KWP2000Command {
    /// Start or change ECU diagnostic session mode. See [start_diagnostic_session]
    StartDiagnosticSession,
    /// Reset the ECU. See [ecu_reset]
    ECUReset,
    /// Clears diagnostic information stored on the ECU. See [clear_diagnostic_information]
    ClearDiagnosticInformation,
    /// Reads snapshot data of DTCs stored on the ECU. See [read_status_of_dtc]
    ReadStatusOfDiagnosticTroubleCodes,
    /// Reads DTCs stored on the ECU. See [read_dtc_by_status]
    ReadDiagnosticTroubleCodesByStatus,
    /// Reads ECU identification data. See [read_ecu_identification]
    ReadECUIdentification,
    /// Reads data from the ECU using a local identifier. See [read_data_by_local_id]
    ReadDataByLocalIdentifier,
    /// Reads data from the ECU using a unique identifier. See [read_data_by_identifier]
    ReadDataByIdentifier,
    /// Reads memory from the ECU by address. See [read_memory_by_address]
    ReadMemoryByAddress,
    /// Security access functions. See [security_access]
    SecurityAccess,
    /// Disables normal CAN message transmission from an ECU. See [enable_normal_message_transmission]
    DisableNormalMessageTransmission,
    /// Enables normal CAN message transmission from an ECU. See [disable_normal_message_transmission]
    EnableNormalMessageTransmission,
    ///
    DynamicallyDefineLocalIdentifier,
    ///
    WriteDataByIdentifier,
    ///
    InputOutputControlByLocalIdentifier,
    /// Starts a ECU routine given a local identifier. See [routine]
    StartRoutineByLocalIdentifier,
    /// Stops a ECU routine given a local identifier. See [routine]
    StopRoutineByLocalIdentifier,
    /// requests results of an executed routine given a local identifier. See [routine]
    RequestRoutineResultsByLocalIdentifier,
    ///
    RequestDownload,
    ///
    RequestUpload,
    ///
    TransferData,
    ///
    RequestTransferExit,
    ///
    WriteDataByLocalIdentifier,
    ///
    WriteMemoryByAddress,
    /// Tester present message. [Kwp2000DiagnosticServer] will automatically send this,
    /// so no need to manually create a message with this SID
    TesterPresent,
    ///
    ControlDTCSettings,
    ///
    ResponseOnEvent,
    /// Custom KWP2000 SID not part of the official specification
    CustomSid(u8)
}

impl From<u8> for KWP2000Command {
    fn from(sid: u8) -> Self {
        match sid {
            0x10 => KWP2000Command::StartDiagnosticSession,
            0x11 => KWP2000Command::ECUReset,
            0x14 => KWP2000Command::ClearDiagnosticInformation,
            0x17 => KWP2000Command::ReadStatusOfDiagnosticTroubleCodes,
            0x18 => KWP2000Command::ReadDiagnosticTroubleCodesByStatus,
            0x1A => KWP2000Command::ReadECUIdentification,
            0x21 => KWP2000Command::ReadDataByLocalIdentifier,
            0x22 => KWP2000Command::ReadDataByIdentifier,
            0x23 => KWP2000Command::ReadMemoryByAddress,
            0x27 => KWP2000Command::SecurityAccess,
            0x28 => KWP2000Command::DisableNormalMessageTransmission,
            0x29 => KWP2000Command::EnableNormalMessageTransmission,
            0x2C => KWP2000Command::DynamicallyDefineLocalIdentifier,
            0x2E => KWP2000Command::WriteDataByIdentifier,
            0x30 => KWP2000Command::InputOutputControlByLocalIdentifier,
            0x31 => KWP2000Command::StartRoutineByLocalIdentifier,
            0x32 => KWP2000Command::StopRoutineByLocalIdentifier,
            0x33 => KWP2000Command::RequestRoutineResultsByLocalIdentifier,
            0x34 => KWP2000Command::RequestDownload,
            0x35 => KWP2000Command::RequestUpload,
            0x36 => KWP2000Command::TransferData,
            0x37 => KWP2000Command::RequestTransferExit,
            0x3B => KWP2000Command::WriteDataByLocalIdentifier,
            0x3D => KWP2000Command::WriteMemoryByAddress,
            0x3E => KWP2000Command::TesterPresent,
            0x85 => KWP2000Command::ControlDTCSettings,
            0x86 => KWP2000Command::ResponseOnEvent,
            s => KWP2000Command::CustomSid(s),
        }
    }
}

impl From<KWP2000Command> for u8 {
    fn from(cmd: KWP2000Command) -> Self {
        match cmd {
            KWP2000Command::StartDiagnosticSession => 0x10,
            KWP2000Command::ECUReset => 0x11,
            KWP2000Command::ClearDiagnosticInformation => 0x14,
            KWP2000Command::ReadStatusOfDiagnosticTroubleCodes => 0x17,
            KWP2000Command::ReadDiagnosticTroubleCodesByStatus => 0x18,
            KWP2000Command::ReadECUIdentification => 0x1A,
            KWP2000Command::ReadDataByLocalIdentifier => 0x21,
            KWP2000Command::ReadDataByIdentifier => 0x22,
            KWP2000Command::ReadMemoryByAddress => 0x23,
            KWP2000Command::SecurityAccess => 0x27,
            KWP2000Command::DisableNormalMessageTransmission => 0x28,
            KWP2000Command::EnableNormalMessageTransmission => 0x29,
            KWP2000Command::DynamicallyDefineLocalIdentifier => 0x2C,
            KWP2000Command::WriteDataByIdentifier => 0x2E,
            KWP2000Command::InputOutputControlByLocalIdentifier => 0x30,
            KWP2000Command::StartRoutineByLocalIdentifier => 0x31,
            KWP2000Command::StopRoutineByLocalIdentifier => 0x32,
            KWP2000Command::RequestRoutineResultsByLocalIdentifier => 0x33,
            KWP2000Command::RequestDownload => 0x34,
            KWP2000Command::RequestUpload => 0x35,
            KWP2000Command::TransferData => 0x36,
            KWP2000Command::RequestTransferExit => 0x37,
            KWP2000Command::WriteDataByLocalIdentifier => 0x3B,
            KWP2000Command::WriteMemoryByAddress => 0x3D,
            KWP2000Command::TesterPresent => 0x3E,
            KWP2000Command::ControlDTCSettings => 0x85,
            KWP2000Command::ResponseOnEvent => 0x86,
            KWP2000Command::CustomSid(s) => s,
        }
    }
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

fn lookup_kwp_nrc(x: u8) -> String {
    format!("{:?}", KWP2000Error::from(x))
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
        b.push(u8::from(sid));
        b.extend_from_slice(args);
        Self {
            bytes: b,
            response_required: need_response,
        }
    }

    pub (crate) fn from_raw(s: &[u8], response_required: bool) -> Self {
        Self {
            bytes: s.to_vec(),
            response_required
        }
    }

    /// Returns the KWP2000 Service ID of the command
    pub fn get_kwp_sid(&self) -> KWP2000Command {
        self.bytes[0].into()
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
    pub fn new_over_iso_tp<C, E>(
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
                            lookup_kwp_nrc
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
                            lookup_kwp_nrc
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
                        helpers::perform_cmd(addr, &cmd, &settings, &mut server_channel, 0x78, 0x21, lookup_kwp_nrc)
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

    /// Returns the current settings used by the KWP2000 Server
    pub fn get_settings(&self) -> Kwp2000ServerOptions {
        self.settings
    }

    /// Internal command for sending KWP2000 payload to the ECU
    fn exec_command(&mut self, cmd: Kwp2000Cmd) -> DiagServerResult<Vec<u8>> {
        match self.tx.send(cmd) {
            Ok(_) => self.rx.recv().unwrap_or(Err(DiagError::ServerNotRunning)),
            Err(_) => Err(DiagError::ServerNotRunning), // Server must have crashed!
        }
    }
}

impl DiagnosticServer<KWP2000Command> for Kwp2000DiagnosticServer {

    /// Send a command to the ECU, and receive its response
    ///
    /// ## Parameters
    /// * sid - The Service ID of the command
    /// * args - The arguments for the service
    ///
    /// ## Returns
    /// If the function is successful, and the ECU responds with an OK response (Containing data),
    /// then the full ECU response is returned. The response will begin with the sid + 0x40
    fn execute_command_with_response(
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
                        if let DiagError::ECUError {code, def} = e {
                            return Err(DiagError::ECUError {code, def}); // ECU Error. Sending again won't help.
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
    fn execute_command(&mut self, sid: KWP2000Command, args: &[u8]) -> DiagServerResult<()> {
        let cmd = Kwp2000Cmd::new(sid, args, false);
        self.exec_command(cmd).map(|_| ())
    }

    /// Sends an arbitrary byte array to the ECU, and does not query response from the ECU
    fn send_byte_array(&mut self, arr: &[u8]) -> DiagServerResult<()> {
        let cmd = Kwp2000Cmd::from_raw(arr, false);
        self.exec_command(cmd).map(|_| ())
    }

    /// Sends an arbitrary byte array to the ECU, and polls for the ECU's response
    fn send_byte_array_with_response(&mut self, arr: &[u8]) -> DiagServerResult<Vec<u8>> {
        let cmd = Kwp2000Cmd::from_raw(arr, true);
        self.exec_command(cmd)
    }

    /// Sets the command retry counter
    fn set_repeat_count(&mut self, count: u32) {
        self.repeat_count = count
    }

    /// Sets the command retry interval
    fn set_repeat_interval_count(&mut self, interval_ms: u32) {
        self.repeat_interval = std::time::Duration::from_millis(interval_ms as u64)
    }

    /// Returns true if the internal KWP2000 Server is running
    fn is_server_running(&self) -> bool {
        self.server_running.load(Ordering::Relaxed)
    }
}

/// Returns the KWP2000 error from a given error code
pub fn get_description_of_ecu_error(error: u8) -> KWP2000Error {
    error.into()
}
