//! Module for UDS (Unified diagnostic services - ISO14229)
//!
//! Theoretically, this module should be compliant with any ECU which implements
//! UDS (Typically any ECU produced after 2006 supports this)

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, RwLock,
    },
    time::Instant,
};

use crate::{
    channel::IsoTPChannel, channel::IsoTPSettings, dtc::DTCFormatType, helpers, BaseServerPayload,
    BaseServerSettings, DiagError, DiagServerResult, DiagnosticServer, ServerEvent,
    ServerEventHandler,
};

mod access_timing_parameter;
mod clear_diagnostic_information;
mod communication_control;
mod diagnostic_session_control;
mod ecu_reset;
mod read_dtc_information;
mod scaling_data;
mod security_access;

pub use access_timing_parameter::*;
pub use clear_diagnostic_information::*;
pub use communication_control::*;
pub use diagnostic_session_control::*;
pub use ecu_reset::*;
pub use read_dtc_information::*;
pub use scaling_data::*;
pub use security_access::*;

pub use auto_uds::{Command as UDSCommand, UdsError as UDSError};

fn lookup_uds_nrc(x: u8) -> String {
    format!("{:?}", UDSError::from(x))
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
        b.push(sid.into());
        b.extend_from_slice(args);
        Self {
            bytes: b,
            response_required: need_response,
        }
    }

    pub(crate) fn from_raw(r: &[u8], response_required: bool) -> Self {
        Self {
            bytes: r.to_vec(),
            response_required,
        }
    }

    /// Returns the UDS Service ID of the command
    pub fn get_uds_sid(&self) -> UDSCommand {
        self.bytes[0].into()
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

impl ServerEventHandler<UDSSessionType> for UdsVoidHandler {
    #[inline(always)]
    fn on_event(&mut self, _e: ServerEvent<UDSSessionType>) {}
}

#[derive(Debug)]
/// UDS Diagnostic server
pub struct UdsDiagnosticServer {
    server_running: Arc<AtomicBool>,
    settings: Arc<RwLock<UdsServerOptions>>,
    tx: mpsc::Sender<UdsCmd>,
    rx: mpsc::Receiver<DiagServerResult<Vec<u8>>>,
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
        setting: UdsServerOptions,
        mut server_channel: C,
        channel_cfg: IsoTPSettings,
        mut event_handler: E,
    ) -> DiagServerResult<Self>
    where
        C: IsoTPChannel + 'static,
        E: ServerEventHandler<UDSSessionType> + 'static,
    {
        server_channel.set_iso_tp_cfg(channel_cfg)?;
        server_channel.set_ids(setting.send_id, setting.recv_id)?;
        server_channel.open()?;

        let settings_ref = Arc::new(RwLock::new(setting));

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_t = is_running.clone();

        let (tx_cmd, rx_cmd) = mpsc::channel::<UdsCmd>();
        let (tx_res, rx_res) = mpsc::channel::<DiagServerResult<Vec<u8>>>();

        let settings_ref_clone = settings_ref.clone();
        std::thread::spawn(move || {
            let mut send_tester_present = false;
            let mut last_tester_present_time: Instant = Instant::now();

            event_handler.on_event(ServerEvent::ServerStart);

            loop {
                if !is_running_t.load(Ordering::Relaxed) {
                    break;
                }

                if let Ok(cmd) = rx_cmd.try_recv() {
                    event_handler.on_event(ServerEvent::Request(cmd.to_bytes()));
                    // We have an incoming command
                    if cmd.get_uds_sid() == UDSCommand::DiagnosticSessionControl {
                        // Session change! Handle this differently
                        match helpers::perform_cmd(
                            setting.send_id,
                            &cmd,
                            &settings_ref_clone.read().unwrap().clone(),
                            &mut server_channel,
                            0x21,
                            lookup_uds_nrc,
                        ) {
                            // 0x78 - Response correctly received, response pending
                            Ok(res) => {
                                // Set server session type
                                if cmd.bytes[1] == u8::from(UDSSessionType::Default) {
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
                            setting.send_id,
                            &cmd,
                            &settings_ref_clone.read().unwrap().clone(),
                            &mut server_channel,
                            0x21,
                            lookup_uds_nrc,
                        );
                        event_handler.on_event(ServerEvent::Response(&res));
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
                        >= setting.tester_present_interval_ms
                {
                    // Send tester present message
                    let cmd = UdsCmd::new(UDSCommand::TesterPresent, &[0x00], true);
                    let addr = match setting.global_tp_id {
                        0 => setting.send_id,
                        x => x,
                    };

                    if let Err(e) = helpers::perform_cmd(
                        addr,
                        &cmd,
                        &settings_ref_clone.read().unwrap().clone(),
                        &mut server_channel,
                        0x21,
                        lookup_uds_nrc,
                    ) {
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
            settings: settings_ref,
            repeat_count: 3,
            repeat_interval: std::time::Duration::from_millis(1000),
            dtc_format: None,
        })
    }

    /// Returns the current settings used by the UDS Server
    pub fn get_settings(&self) -> UdsServerOptions {
        self.settings.read().unwrap().clone()
    }

    /// Internal command for sending UDS payload to the ECU
    fn exec_command(&mut self, cmd: UdsCmd) -> DiagServerResult<Vec<u8>> {
        match self.tx.send(cmd) {
            Ok(_) => self.rx.recv().unwrap_or(Err(DiagError::ServerNotRunning)),
            Err(_) => Err(DiagError::ServerNotRunning), // Server must have crashed!
        }
    }
}

impl DiagnosticServer<UDSCommand> for UdsDiagnosticServer {
    fn is_server_running(&self) -> bool {
        self.server_running.load(Ordering::Relaxed)
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
    fn execute_command_with_response(
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
                        if let DiagError::ECUError { code, def } = e {
                            return Err(DiagError::ECUError { code, def }); // ECU Error. Sending again won't help.
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
    fn execute_command(&mut self, sid: UDSCommand, args: &[u8]) -> DiagServerResult<()> {
        let cmd = UdsCmd::new(sid, args, false);
        self.exec_command(cmd).map(|_| ())
    }

    /// Sets the command retry counter
    fn set_repeat_count(&mut self, count: u32) {
        self.repeat_count = count
    }

    /// Sets the command retry interval
    fn set_repeat_interval_count(&mut self, interval_ms: u32) {
        self.repeat_interval = std::time::Duration::from_millis(interval_ms as u64)
    }

    /// Sends an arbitrary byte array to the ECU, and does not query response from the ECU
    fn send_byte_array(&mut self, arr: &[u8]) -> DiagServerResult<()> {
        let cmd = UdsCmd::from_raw(arr, false);
        self.exec_command(cmd).map(|_| ())
    }

    /// Sends an arbitrary byte array to the ECU, and polls for the ECU's response
    fn send_byte_array_with_response(&mut self, arr: &[u8]) -> DiagServerResult<Vec<u8>> {
        let cmd = UdsCmd::from_raw(arr, true);
        self.exec_command(cmd)
    }

    /// Sets read and write timeouts
    fn set_rw_timeout(&mut self, read_timeout_ms: u32, write_timeout_ms: u32) {
        let mut lock = self.settings.write().unwrap();
        lock.read_timeout_ms = read_timeout_ms;
        lock.write_timeout_ms = write_timeout_ms;
    }

    /// Get command response read timeout
    fn get_read_timeout(&self) -> u32 {
        self.settings.read().unwrap().read_timeout_ms
    }
    /// Gets command write timeout
    fn get_write_timeout(&self) -> u32 {
        self.settings.read().unwrap().write_timeout_ms
    }
}

/// Returns the [UDSError] from a matching input byte.
/// The error byte provided MUST come from [DiagError::ECUError]
pub fn get_description_of_ecu_error(error: u8) -> UDSError {
    error.into()
}

impl Drop for UdsDiagnosticServer {
    fn drop(&mut self) {
        self.server_running.store(false, Ordering::Relaxed); // Stop server
    }
}
