//! Dynamic diagnostic session helper
//!
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
        Mutex,
    },
    time::Duration,
};

use std::{
    sync::{mpsc, Arc, RwLock},
    time::Instant,
};

use crate::{
    channel::{ChannelResult, IsoTPChannel, IsoTPSettings},
    DiagError, DiagServerResult,
};

#[derive(Debug)]
/// Diagnostic server responses
pub enum DiagServerRx {
    /// Positive ECU response
    EcuResponse(Vec<u8>),
    /// ECU error
    EcuError {
        /// Raw NRC byte
        b: u8,
        /// NRC description
        desc: String,
    },
    /// ECU is busy, please wait
    EcuBusy,
    /// Request message transmit result
    SendState {
        /// Data that was sent to be transmitted
        p: Vec<u8>,
        /// The send result of the data transmission
        r: DiagServerResult<()>,
    },
    /// Receive response error
    RecvError(DiagError),
}

impl DiagServerRx {
    const fn is_err(&self) -> bool {
        match self {
            DiagServerRx::EcuResponse(_) => false,
            DiagServerRx::SendState { p: _, r } => r.is_err(),
            _ => true,
        }
    }

    const fn is_ok(&self) -> bool {
        !self.is_err()
    }
}

/// ECU Negative response code trait
pub trait EcuNRC: From<u8> {
    /// Description of the NRC
    fn desc(&self) -> String;
    /// Returns true if the NRC implies the ECU is busy, and the Diagnostic server
    /// should wait for the ECU's real response
    fn is_ecu_busy(&self) -> bool;
    /// Returns true if the NRC means the ECU is in the wrong diagnostic
    /// mode to process the current service
    fn is_wrong_diag_mode(&self) -> bool;
    /// Returns true if the ECU has asked the diagnostic server to repeat the request message
    fn is_repeat_request(&self) -> bool;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
/// Server read/write timeout configuration
pub struct TimeoutConfig {
    /// Read timeout
    pub read_timeout_ms: u32,
    /// Write timeout
    pub write_timeout_ms: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
/// Basic diagnostic server options
pub struct DiagServerBasicOptions {
    /// ECU Send ID
    pub send_id: u32,
    /// ECU Receive ID
    pub recv_id: u32,
    /// Timeout config
    pub timeout_cfg: TimeoutConfig,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Advanced diagnostic server options
pub struct DiagServerAdvancedOptions {
    /// Optional global address to send tester-present messages to
    /// Set to 0 if not in use
    pub global_tp_id: u32,
    /// Tester present minimum send interval in ms.
    pub tester_present_interval_ms: u32,
    /// Configures if the diagnostic server will poll for a response from tester present.
    pub tester_present_require_response: bool,
    /// Session control uses global_tp_id if specified
    /// If `global_tp_id` is set to 0, then this value is ignored.
    ///
    /// IMPORTANT: This can set your ENTIRE vehicle network into diagnostic
    /// session mode, so be very careful doing this!
    pub global_session_control: bool,
    /// Extended ISO-TP Address (Only for tester present)
    /// Some ECUs may require this in combination with a global tp ID
    pub tp_ext_id: Option<u8>,
    /// Cooldown period in MS after receiving a response from an ECU before sending a request.
    /// This is useful for some slower ECUs
    pub command_cooldown_ms: u32,
}

#[derive(Debug)]
/// Diagnostic server event, used when using a [DiagServerLogger]
pub enum ServerEvent {
    /// Diag server started
    ServerStart,
    /// Diag server stopped
    ServerExit,
    /// Sent payload to ECU
    BytesSendState(u32, Vec<u8>, ChannelResult<()>),
    /// Recv payload from ECU
    BytesRecvState(u32, ChannelResult<Vec<u8>>),
}

/// Diag server logger
pub trait DiagServerLogger: Clone + Send + Sync {
    /// When a diagnostic server event happens
    fn on_event(&self, _evt: ServerEvent) {}
}

#[derive(Debug, Copy, Clone)]
/// Diag server basic logger (Use this if no logger is to be used in your application)
pub struct DiagServerEmptyLogger {}

impl DiagServerLogger for DiagServerEmptyLogger {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Diagnostic session mode
pub struct DiagSessionMode {
    /// Session mode ID
    pub id: u8,
    /// Tester present required?
    pub tp_require: bool,
    /// Alias for its name (For logging only)
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Diagnostic request payload
pub struct DiagPayload {
    /// Service ID
    sid: u8,
    /// parameter ID and rest of the message
    data: Vec<u8>,
}

impl DiagPayload {
    /// Converts DiagPayload to a byte array to be sent to the ECU
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut r = vec![self.sid];
        r.extend_from_slice(&self.data);
        r
    }

    /// Creates a new DiagPayload
    pub fn new(sid: u8, data: &[u8]) -> Self {
        Self {
            sid,
            data: data.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Diagnostic server request action
pub enum DiagAction {
    /// Set session mode
    SetSessionMode(DiagSessionMode),
    /// ECU Reset message (On completion, ECU will be back in default diag mode)
    EcuReset,
    /// Other request
    Other {
        /// Service ID
        sid: u8,
        /// PID and data
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Diagnostic request param info
struct DiagTxPayload {
    /// Raw payload to send to the ECU
    pub payload: Vec<u8>,
    /// Should the payload require a response from the ECU?
    pub response_require: bool,
}

/// Diagnostic protocol description trait
pub trait DiagProtocol<NRC>: Send + Sync
where
    NRC: EcuNRC,
{
    /// Returns the alias to the ECU 'default' diagnostic session mode
    /// Returns None if there is no session type control in the protocol
    /// (For example basic OBD2)
    fn get_basic_session_mode(&self) -> Option<DiagSessionMode>;
    /// Name of the diagnostic protocol
    fn get_protocol_name(&self) -> &'static str;
    /// Process a byte array into a command
    fn process_req_payload(&self, payload: &[u8]) -> DiagAction;
    /// Creates a session mod req message
    fn make_session_control_msg(&self, mode: &DiagSessionMode) -> Vec<u8>;
    /// Generate the tester present message (If required)
    fn create_tp_msg(response_required: bool) -> DiagPayload;
    /// Processes the ECU response, and checks to see if it is a positive or negative response,
    /// this includes checking to see if the ECU is in a waiting state
    fn process_ecu_response(r: &[u8]) -> Result<Vec<u8>, (u8, NRC)>;
    /// Gets a hashmap of available diagnostic session modes
    fn get_diagnostic_session_list(&self) -> HashMap<u8, DiagSessionMode>;
    /// Registers a new custom diagnostic session mode
    fn register_session_type(&mut self, session: DiagSessionMode);
}

// Callbacks

/// Transmit data callback
pub type TxCallbackFn = dyn Fn(&[u8]);
/// ECU Waiting callback
pub type EcuWaitCallbackFn = dyn Fn();

/// Dynamic diagnostic session
///
/// This is used if a target ECU has an unknown diagnostic protocol.
///
/// This also contains some useful wrappers for basic functions such as
/// reading and clearing error codes.
pub struct DynamicDiagSession {
    sender: Mutex<Sender<DiagTxPayload>>,
    receiver: Receiver<DiagServerRx>,
    waiting_hook: Box<EcuWaitCallbackFn>,
    on_send_complete_hook: Box<TxCallbackFn>,
    connected: Arc<AtomicBool>,
    current_diag_mode: Arc<RwLock<Option<DiagSessionMode>>>,
    running: Arc<AtomicBool>,
}

unsafe impl Sync for DynamicDiagSession {}
unsafe impl Send for DynamicDiagSession {}

impl std::fmt::Debug for DynamicDiagSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicDiagSession")
            .field("sender", &self.sender)
            .field("receiver", &self.receiver)
            .finish()
    }
}

impl DynamicDiagSession {
    /// Creates a new diagnostic server with a given protocol and NRC format
    /// over an ISO-TP connection
    #[allow(unused_must_use, unused_assignments)]
    pub fn new_over_iso_tp<P, NRC, L>(
        protocol: P,
        mut channel: Box<dyn IsoTPChannel>,
        channel_cfg: IsoTPSettings,
        basic_opts: DiagServerBasicOptions,
        advanced_opts: Option<DiagServerAdvancedOptions>,
        mut logger: L,
    ) -> DiagServerResult<Self>
    where
        P: DiagProtocol<NRC> + 'static,
        NRC: EcuNRC,
        L: DiagServerLogger + 'static,
    {
        // Create iso tp channel using provided HW interface. If this fails, we cannot setup KWP or UDS session!
        channel.set_iso_tp_cfg(channel_cfg)?;
        channel.set_ids(basic_opts.send_id, basic_opts.recv_id)?;
        channel.open()?;

        let mut current_session_mode = protocol.get_basic_session_mode();
        let mut requested_session_mode = protocol.get_basic_session_mode();
        if current_session_mode.is_none() && advanced_opts.is_some() {
            log::warn!(
                "Session mode is None but advanced opts was specified. Ignoring advanced opts"
            );
        }
        let session_control = current_session_mode.is_some() && advanced_opts.is_some();
        let (tx_req, rx_req) = mpsc::channel::<DiagTxPayload>();
        let (mut tx_resp, rx_resp) = mpsc::channel::<DiagServerRx>();
        let is_connected = Arc::new(AtomicBool::new(true));
        let is_connected_inner = is_connected.clone();

        let noti_session_mode = Arc::new(RwLock::new(current_session_mode.clone()));
        let noti_session_mode_t = noti_session_mode.clone();

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_c = is_running.clone();
        let cooldown = advanced_opts.map(|x| x.command_cooldown_ms).unwrap_or(0) as u128;
        std::thread::spawn(move || {
            let mut last_tp_time = Instant::now();
            let mut last_cmd_time = Instant::now();
            logger.on_event(ServerEvent::ServerStart);
            let rx_addr = basic_opts.recv_id;
            while is_running.load(Ordering::Relaxed) {
                // Incomming ECU request (Check only after cooldown)
                let mut do_cmd = false;
                if cooldown == 0 || last_cmd_time.elapsed().as_millis() >= cooldown {
                    if let Ok(req) = rx_req.recv_timeout(Duration::from_millis(100)) {
                        do_cmd = true;
                        let mut tx_addr = basic_opts.send_id;
                        match protocol.process_req_payload(&req.payload) {
                            DiagAction::SetSessionMode(mode) => {
                                let needs_response = true;
                                let ext_id = None;
                                let res = send_recv_ecu_req::<P, NRC, L>(
                                    tx_addr,
                                    rx_addr,
                                    ext_id,
                                    &req.payload,
                                    needs_response,
                                    Some(&mut tx_resp),
                                    basic_opts,
                                    0,
                                    &mut channel,
                                    &is_connected_inner,
                                    &mut logger,
                                );
                                if res.is_ok() {
                                    // Send OK! We can set diag mode in the server side
                                    requested_session_mode = Some(mode);
                                    *noti_session_mode_t.write().unwrap() =
                                        requested_session_mode.clone();
                                    last_tp_time = Instant::now();
                                    last_cmd_time = Instant::now();
                                }
                                tx_resp.send(res);
                            }
                            DiagAction::EcuReset => {
                                let res = send_recv_ecu_req::<P, NRC, L>(
                                    tx_addr,
                                    rx_addr,
                                    None,
                                    &req.payload,
                                    req.response_require,
                                    Some(&mut tx_resp),
                                    basic_opts,
                                    0,
                                    &mut channel,
                                    &is_connected_inner,
                                    &mut logger,
                                );
                                if res.is_ok() {
                                    log::debug!("ECU Reset detected. Setting default session mode");
                                    // Send OK! We have to set default session mode as the ECU has been rebooted
                                    // Internally, the 'current session' does not change, as diag server will try on the next
                                    // request to change modes back
                                    *noti_session_mode_t.write().unwrap() =
                                        protocol.get_basic_session_mode();
                                    current_session_mode = protocol.get_basic_session_mode();
                                    std::thread::sleep(Duration::from_millis(500)); // Await ECU to reboot - TODO. Maybe we should let this be configured?
                                    last_cmd_time = Instant::now();
                                }
                                tx_resp.send(res);
                            }
                            _ => {
                                let mut resp = send_recv_ecu_req::<P, NRC, L>(
                                    tx_addr,
                                    rx_addr,
                                    None,
                                    &req.payload,
                                    req.response_require,
                                    Some(&mut tx_resp),
                                    basic_opts,
                                    0,
                                    &mut channel,
                                    &is_connected_inner,
                                    &mut logger,
                                );
                                if let DiagServerRx::EcuError { b, desc: _ } = &resp {
                                    if NRC::from(*b).is_wrong_diag_mode() {
                                        log::debug!("Trying to switch ECU modes");
                                        // Wrong diag mode. We need to see if we need to change modes
                                        // Switch modes!
                                        tx_resp.send(DiagServerRx::EcuBusy); // Until we have a hook for this specific scenerio
                                                                             // Now create new diag server request message
                                        let mut needs_response = true;
                                        let mut ext_id = None;
                                        if let Some(adv) = advanced_opts {
                                            if adv.global_session_control && adv.global_tp_id != 0 {
                                                tx_addr = adv.global_tp_id;
                                                ext_id = adv.tp_ext_id;
                                                needs_response = false;
                                            } else if adv.global_session_control
                                                && adv.global_tp_id == 0
                                            {
                                                log::warn!("Global session control is enabled but global TP ID is not specified");
                                            }
                                        }
                                        if send_recv_ecu_req::<P, NRC, L>(
                                            tx_addr,
                                            rx_addr,
                                            ext_id,
                                            &protocol.make_session_control_msg(
                                                &current_session_mode.clone().unwrap(),
                                            ),
                                            needs_response,
                                            None, // None, internally handled
                                            basic_opts,
                                            0,
                                            &mut channel,
                                            &is_connected_inner,
                                            &mut logger,
                                        )
                                        .is_ok()
                                        {
                                            log::debug!(
                                                "ECU mode switch OK. Resending the request"
                                            );
                                            *noti_session_mode_t.write().unwrap() =
                                                current_session_mode.clone();
                                            last_tp_time = Instant::now();
                                            // Resend our request
                                            resp = send_recv_ecu_req::<P, NRC, L>(
                                                tx_addr,
                                                rx_addr,
                                                None,
                                                &req.payload,
                                                req.response_require,
                                                Some(&mut tx_resp),
                                                basic_opts,
                                                0,
                                                &mut channel,
                                                &is_connected_inner,
                                                &mut logger,
                                            );
                                        } else {
                                            // Diag session mode req failed. Set session data
                                            *noti_session_mode_t.write().unwrap() =
                                                protocol.get_basic_session_mode();
                                            current_session_mode = protocol.get_basic_session_mode()
                                        }
                                    }
                                } else if let DiagServerRx::EcuResponse(_) = &resp {
                                    last_cmd_time = Instant::now();
                                }
                                tx_resp.send(resp);
                            }
                        }
                    }
                }
                if !do_cmd {
                    if current_session_mode != requested_session_mode {
                        current_session_mode = requested_session_mode.clone();
                    }
                    // Nothing to process, so sleep and/or tester present processing
                    // Logic for handling session control TP present requests
                    if session_control {
                        let c_mode = current_session_mode.clone().unwrap();
                        let aops = advanced_opts.unwrap();
                        if c_mode.tp_require
                            && last_tp_time.elapsed().as_millis() as u32
                                >= aops.tester_present_interval_ms
                        {
                            let tx_payload = P::create_tp_msg(aops.tester_present_require_response);
                            let tx_addr = if aops.global_tp_id != 0 {
                                aops.global_tp_id
                            } else {
                                basic_opts.send_id
                            };
                            if send_recv_ecu_req::<P, NRC, L>(
                                tx_addr,
                                rx_addr,
                                aops.tp_ext_id,
                                &tx_payload.to_bytes(),
                                aops.tester_present_require_response,
                                None,
                                basic_opts,
                                0,
                                &mut channel,
                                &is_connected_inner,
                                &mut logger,
                            )
                            .is_err()
                            {
                                log::warn!("Tester present send failure. Assuming default diag session state");
                                current_session_mode = protocol.get_basic_session_mode();
                                *noti_session_mode_t.write().unwrap() =
                                    current_session_mode.clone();
                            } else {
                                last_tp_time = Instant::now(); // OK, reset the timer
                            }
                        }
                    }
                }
            }
            logger.on_event(ServerEvent::ServerExit);
            // Thread has exited, so tear everything down!
            channel.close().unwrap();
            drop(channel)
        });
        Ok(Self {
            sender: Mutex::new(tx_req),
            receiver: rx_resp,
            waiting_hook: Box::new(|| {}),
            on_send_complete_hook: Box::new(|_| {}),
            connected: is_connected,
            current_diag_mode: noti_session_mode,
            running: is_running_c,
        })
    }

    /// Send a command
    pub fn send_command<T: Into<u8>>(&self, cmd: T, args: &[u8]) -> DiagServerResult<()> {
        let mut r = vec![cmd.into()];
        r.extend_from_slice(args);
        let lock = self.sender.lock().unwrap();
        self.internal_send_byte_array(&r, &lock, false)
    }

    /// Send a byte array
    pub fn send_byte_array(&self, p: &[u8]) -> DiagServerResult<()> {
        let lock = self.sender.lock().unwrap();
        self.internal_send_byte_array(p, &lock, false)
    }

    fn internal_send_byte_array(
        &self,
        p: &[u8],
        sender: &Sender<DiagTxPayload>,
        resp_require: bool,
    ) -> DiagServerResult<()> {
        self.clear_rx_queue();
        sender
            .send(DiagTxPayload {
                payload: p.to_vec(),
                response_require: resp_require,
            })
            .unwrap();
        loop {
            if let DiagServerRx::SendState { p: _, r } = self.receiver.recv().unwrap() {
                return r;
            }
        }
    }

    /// Send a command to the ECU and await its response
    pub fn send_command_with_response<T: Into<u8>>(
        &self,
        cmd: T,
        args: &[u8],
    ) -> DiagServerResult<Vec<u8>> {
        let mut r = vec![cmd.into()];
        r.extend_from_slice(args);
        self.send_byte_array_with_response(&r)
    }

    /// Send bytes to the ECU and await its response
    /// ## Params
    /// * p - Raw byte array to send
    /// * on_ecu_waiting_hook - Callback to call when the ECU responds with ResponsePending. Can be used to update a programs state
    /// such that the user is aware the ECU is just processing the request
    pub fn send_byte_array_with_response(&self, p: &[u8]) -> DiagServerResult<Vec<u8>> {
        let lock = self.sender.lock().unwrap();
        self.internal_send_byte_array(p, &lock, true)?;
        (self.on_send_complete_hook)(p);
        loop {
            match self.receiver.recv().unwrap() {
                DiagServerRx::EcuResponse(r) => return Ok(r),
                DiagServerRx::EcuError { b, desc } => {
                    return Err(DiagError::ECUError {
                        code: b,
                        def: Some(desc),
                    })
                }
                DiagServerRx::EcuBusy => (self.waiting_hook)(),
                DiagServerRx::SendState { p, r } => match r {
                    Ok(_) => (self.on_send_complete_hook)(&p),
                    Err(e) => return Err(e),
                },
                DiagServerRx::RecvError(e) => return Err(e),
            }
        }
    }

    /// Returns true only if a hardware failure has resulted in the ECU
    /// disconnecting from the diagnostic server.
    pub fn is_ecu_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Register a hook that will be called whenever the ECU has replyed with
    /// a 'Busy' or 'Please wait' response
    pub fn register_waiting_hook<F: Fn() + 'static>(&mut self, hook: F) {
        self.waiting_hook = Box::new(hook)
    }

    /// Register a hook that will be called whenever data has been sent out to the ECU
    /// successfully
    pub fn register_send_complete_hook<F: Fn(&[u8]) + 'static>(&mut self, hook: F) {
        self.on_send_complete_hook = Box::new(hook)
    }

    /// Returns the current diagnostic session mode that the ECU is in
    pub fn get_current_diag_mode(&self) -> Option<DiagSessionMode> {
        self.current_diag_mode.read().unwrap().clone()
    }

    fn clear_rx_queue(&self) {
        while self.receiver.try_recv().is_ok() {}
    }
}

impl Drop for DynamicDiagSession {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed)
    }
}

fn send_recv_ecu_req<P, NRC, L>(
    tx_addr: u32,
    rx_addr: u32,
    ext_id: Option<u8>,
    payload: &[u8], // If empty, we are only reading
    needs_response: bool,
    tx_resp: Option<&mut Sender<DiagServerRx>>,
    basic_opts: DiagServerBasicOptions,
    cooldown: u32,
    channel: &mut Box<dyn IsoTPChannel>,
    connect_state: &AtomicBool,
    logger: &mut L,
) -> DiagServerRx
where
    P: DiagProtocol<NRC>,
    NRC: EcuNRC,
    L: DiagServerLogger,
{
    // Send the request, and transmit the send state!
    let mut res: ChannelResult<()> = Ok(());
    if !payload.is_empty() {
        // We need to write some bytes
        log::debug!("Sending req to ECU: {payload:02X?}");
        res = channel
            .clear_tx_buffer()
            .and_then(|_| channel.clear_rx_buffer())
            .and_then(|_| {
                channel.write_bytes(
                    tx_addr,
                    ext_id,
                    payload,
                    basic_opts.timeout_cfg.write_timeout_ms,
                )
            })
    }
    match res {
        Ok(_) => {
            if !payload.is_empty() {
                logger.on_event(ServerEvent::BytesSendState(
                    tx_addr,
                    payload.to_vec(),
                    Ok(()),
                ));
            }
            if needs_response {
                log::debug!("Sending OK, awaiting response from ECU");
                // Notify sending has completed, we will now poll for the ECUs response!
                if let Some(s) = &tx_resp {
                    s.send(DiagServerRx::SendState {
                        p: payload.to_vec(),
                        r: Ok(()),
                    })
                    .unwrap();
                }
                // Now poll for the ECU's response
                let r_state = channel.read_bytes(basic_opts.timeout_cfg.read_timeout_ms);
                logger.on_event(ServerEvent::BytesRecvState(rx_addr, r_state.clone()));
                match r_state {
                    Err(e) => {
                        log::error!("Error reading from channel. Request was {payload:02X?}");
                        connect_state.store(false, Ordering::Relaxed);
                        // Final error
                        DiagServerRx::RecvError(e.into())
                    }
                    Ok(bytes) => {
                        log::debug!("ECU Response: {bytes:02X?}");
                        let parsed_response = P::process_ecu_response(&bytes);
                        connect_state.store(true, Ordering::Relaxed);
                        match parsed_response {
                            Ok(pos_result) => {
                                log::debug!("ECU Response OK!");
                                DiagServerRx::EcuResponse(pos_result)
                            }
                            Err((code, nrc_data)) => {
                                if nrc_data.is_ecu_busy() {
                                    // ECU waiting, so poll again for the response
                                    // to do that, call this function again with no payload
                                    log::debug!("ECU is busy, awaiting response");
                                    send_recv_ecu_req::<P, NRC, L>(
                                        tx_addr,
                                        rx_addr,
                                        ext_id,
                                        &[],
                                        needs_response,
                                        tx_resp,
                                        basic_opts,
                                        cooldown,
                                        channel,
                                        connect_state,
                                        logger,
                                    )
                                } else if nrc_data.is_repeat_request() {
                                    // ECU wants us to ask again, so we wait a little bit, then call ourselves again
                                    log::debug!("ECU has asked for a repeat of the request");
                                    std::thread::sleep(Duration::from_millis(cooldown.into()));
                                    send_recv_ecu_req::<P, NRC, L>(
                                        tx_addr,
                                        rx_addr,
                                        ext_id,
                                        payload,
                                        needs_response,
                                        tx_resp,
                                        basic_opts,
                                        cooldown,
                                        channel,
                                        connect_state,
                                        logger,
                                    )
                                } else {
                                    // Unhandled NRC
                                    log::warn!("ECU Negative response {code:02X?}");
                                    DiagServerRx::EcuError {
                                        b: code,
                                        desc: nrc_data.desc(),
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // Final state. We are done!
                log::debug!("No need to poll ECU response");
                connect_state.store(true, Ordering::Relaxed);
                DiagServerRx::SendState {
                    p: payload.to_vec(),
                    r: Ok(()),
                }
            }
        }
        Err(e) => {
            logger.on_event(ServerEvent::BytesSendState(
                rx_addr,
                payload.to_vec(),
                Err(e.clone()),
            ));
            log::error!("Channel send error: {e}");
            // Final error here at send state :(
            connect_state.store(false, Ordering::Relaxed);
            DiagServerRx::SendState {
                p: payload.to_vec(),
                r: Err(e.into()),
            }
        }
    }
}
