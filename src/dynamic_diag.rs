//! Dynamic diagnostic session helper
//!
use std::{sync::{mpsc::Sender, atomic::{AtomicBool, Ordering}}, time::Duration, collections::HashMap};

use std::{
    sync::{Arc, Mutex, RwLock, mpsc}, time::Instant,
};

use crate::{
    channel::{IsoTPSettings, IsoTPChannel, ChannelResult},
    hardware::Hardware,
    DiagError, DiagServerResult
};

#[derive(Debug)]
pub enum DiagServerRx {
    EcuResponse(Vec<u8>),
    EcuError { b: u8, desc: String },
    EcuBusy,
    SendState(DiagServerResult<()>),
    RecvError(DiagError)
}

impl DiagServerRx {
    const fn is_err(&self) -> bool {
        match self {
            DiagServerRx::EcuResponse(_) => false,
            DiagServerRx::SendState(res) => res.is_err(),
            _ => true
        }
    }

    const fn is_ok(&self) -> bool {
        !self.is_err()
    }
}

pub trait EcuNRC : From<u8> {
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
pub struct TimeoutConfig {
    /// Read timeout
    pub read_timeout_ms: u32,
    /// Write timeout
    pub write_timeout_ms: u32
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
    pub timeout_cfg: TimeoutConfig
}


#[derive(Debug, Copy, Clone)]
#[repr(C)]
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
    pub command_cooldown_ms: u128
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagSessionMode {
    /// Session mode ID
    pub id: u8,
    /// Tester present required?
    pub tp_require: bool,
    /// Alias for its name (For logging only)
    pub name: String
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagPayload {
    /// Service ID
    sid: u8,
    /// parameter ID and rest of the message
    data: Vec<u8>
}

impl DiagPayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut r = vec![self.sid];
        r.extend_from_slice(&self.data);
        r
    }

    pub fn new(sid: u8, data: &[u8]) -> Self {
        Self {
            sid,
            data: data.to_vec()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagAction {
    SetSessionMode(DiagSessionMode),
    Other { sid: u8, data: Vec<u8> }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagTxPayload {
    pub payload: Vec<u8>,
    pub response_require: bool
}

pub trait DiagProtocol<NRC> : Send + Sync where NRC: EcuNRC {
    /// Returns the alias to the ECU 'default' diagnostic session mode
    /// Returns None if there is no session type control in the protocol
    /// (For example basic OBD2)
    fn get_basic_session_mode(&self) -> Option<DiagSessionMode>;
    /// Name of the diagnostic protocol
    fn get_protocol_name(&self) -> &'static str;
    /// Process a byte array into a command
    fn process_req_payload(&self, payload: &[u8]) -> DiagAction;
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

/// Dynamic diagnostic session
///
/// This is used if a target ECU has an unknown diagnostic protocol.
///
/// This also contains some useful wrappers for basic functions such as
/// reading and clearing error codes.
pub struct DynamicDiagSession {
    sender: mpsc::Sender<DiagTxPayload>,
    receiver: mpsc::Receiver<DiagServerRx>,
    waiting_hook: Box<dyn FnMut()>,
    on_send_complete_hook: Box<dyn FnMut()>,
    connected: Arc<AtomicBool>,
    current_diag_mode: Arc<RwLock<Option<DiagSessionMode>>>,
    running: Arc<AtomicBool>
}

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
    pub fn new_over_iso_tp<C, P, NRC>(
        protocol: P,
        hw_device: Arc<Mutex<C>>,
        channel_cfg: IsoTPSettings,
        basic_opts: DiagServerBasicOptions,
        advanced_opts: Option<DiagServerAdvancedOptions>,
    ) -> DiagServerResult<Self>
    where
        C: Hardware + 'static,
        P: DiagProtocol<NRC> + 'static,
        NRC: EcuNRC
    {
        // Create iso tp channel using provided HW interface. If this fails, we cannot setup KWP or UDS session!
        let mut iso_tp_channel = Hardware::create_iso_tp_channel(hw_device.clone())?;
        iso_tp_channel.set_iso_tp_cfg(channel_cfg)?;
        iso_tp_channel.set_ids(basic_opts.send_id, basic_opts.recv_id)?;
        iso_tp_channel.open()?;

        let requested_session_mode = protocol.get_basic_session_mode();
        let mut current_session_mode = protocol.get_basic_session_mode();
        if requested_session_mode.is_none() && advanced_opts.is_some() {
            log::warn!("Session mode is None but advanced opts was specified. Ignoring advanced opts");
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
        std::thread::spawn(move || {
            let mut last_tp_time = Instant::now();
            while is_running.load(Ordering::Relaxed) {
                // Incomming ECU request
                if let Ok(req) = rx_req.recv_timeout(Duration::from_millis(100)) {
                    let mut tx_addr = basic_opts.send_id;
                    match protocol.process_req_payload(&req.payload) {
                        DiagAction::SetSessionMode(mode) => {
                            let mut needs_response = true;
                            let mut ext_id = None;
                            if let Some(adv) = advanced_opts {
                                if adv.global_session_control && adv.global_tp_id != 0 {
                                    tx_addr = adv.global_tp_id;
                                    ext_id = adv.tp_ext_id;
                                    needs_response = false;
                                } else if adv.global_session_control && adv.global_tp_id == 0 {
                                    log::warn!("Global session control is enabled but global TP ID is not specified");
                                }
                            }
                            let res = send_recv_ecu_req(
                                &protocol, 
                                tx_addr, 
                                ext_id, 
                                &req.payload, 
                                needs_response, 
                                Some(&mut tx_resp), 
                                basic_opts, 
                                0, 
                                &mut iso_tp_channel,
                                &is_connected_inner
                            );
                            if res.is_ok() {
                                // Send OK! We can set diag mode in the server side
                                current_session_mode = Some(mode);
                                *noti_session_mode_t.write().unwrap() = current_session_mode.clone();
                                last_tp_time = Instant::now();
                            }
                            tx_resp.send(res);
                        },
                        _ => {
                            let resp = send_recv_ecu_req(
                                &protocol, 
                                tx_addr, 
                                None, 
                                &req.payload, 
                                req.response_require, 
                                Some(&mut tx_resp), 
                                basic_opts, 
                                0, 
                                &mut iso_tp_channel,
                                &is_connected_inner
                            );
                            tx_resp.send(resp);
                        },
                    }
                } else {
                    // Nothing to process, so sleep and/or tester present processing
                    // Logic for handling session control TP present requests
                    if session_control {
                        let c_mode = current_session_mode.clone().unwrap();
                        let aops = advanced_opts.unwrap();
                        if c_mode.tp_require && last_tp_time.elapsed().as_millis() as u32 >= aops.tester_present_interval_ms {
                            let tx_payload = P::create_tp_msg(aops.tester_present_require_response);
                            let tx_addr = if aops.global_tp_id != 0 {
                                aops.global_tp_id
                            } else {
                                basic_opts.send_id
                            };
                            if send_recv_ecu_req(
                                &protocol, 
                                tx_addr, 
                                aops.tp_ext_id, 
                                &tx_payload.to_bytes(), 
                                aops.tester_present_require_response, 
                                None, 
                                basic_opts, 
                                0, 
                                &mut iso_tp_channel,
                                &is_connected_inner
                            ).is_err() {
                                log::warn!("Tester present send failure. Assuming default diag session state");
                                current_session_mode = protocol.get_basic_session_mode();
                                *noti_session_mode_t.write().unwrap() = current_session_mode.clone();
                            } else {
                                last_tp_time = Instant::now(); // OK, reset the timer
                            }
                        }
                    }
                }
            }
            // Thread has exited, so tear everything down!
            iso_tp_channel.close().unwrap();
            drop(iso_tp_channel)
        });
        Ok(Self {
            sender: tx_req,
            receiver: rx_resp,
            waiting_hook: Box::new(||{}),
            on_send_complete_hook: Box::new(||{}),
            connected: is_connected,
            current_diag_mode: noti_session_mode,
            running: is_running_c
        })
    }

    /// Send a command
    pub fn send_command<T: Into<u8>>(&self, cmd: T, args: &[u8]) -> DiagServerResult<()> {
        let mut r = vec![cmd.into()];
        r.extend_from_slice(args);
        self.internal_send_byte_array(&r, false)
    }
    
    /// Send a byte array
    pub fn send_byte_array(&self, p: &[u8]) -> DiagServerResult<()> {
        self.internal_send_byte_array(p, false)
    }

    fn internal_send_byte_array(&self, p: &[u8], resp_require: bool) -> DiagServerResult<()> {
        self.clear_rx_queue();
        self.sender.send(DiagTxPayload { payload: p.to_vec(), response_require: resp_require }).unwrap();
        loop {
            if let DiagServerRx::SendState(res) = self.receiver.recv().unwrap() {
                return res
            }
        }
    }

    /// Send a command to the ECU and await its response
    pub fn send_command_with_response<T: Into<u8>>(&mut self, cmd: T, args: &[u8]) -> DiagServerResult<Vec<u8>> {
        let mut r = vec![cmd.into()];
        r.extend_from_slice(args);
        self.send_byte_array_with_response(&r)
    }

    /// Send bytes to the ECU and await its response
    /// ## Params
    /// * p - Raw byte array to send
    /// * on_ecu_waiting_hook - Callback to call when the ECU responds with ResponsePending. Can be used to update a programs state
    /// such that the user is aware the ECU is just processing the request
    pub fn send_byte_array_with_response(&mut self, p: &[u8]) -> DiagServerResult<Vec<u8>> {
        self.internal_send_byte_array(p, true)?;
        (self.on_send_complete_hook)();
        loop {
            match self.receiver.recv().unwrap() {
                DiagServerRx::EcuResponse(r) => {
                    return Ok(r)
                },
                DiagServerRx::EcuError { b, desc } => {
                    return Err(DiagError::ECUError { code: b, def: Some(desc) })
                },
                DiagServerRx::EcuBusy => {
                    (self.waiting_hook)()
                },
                DiagServerRx::SendState(s) => {
                    log::error!("Multiple send states received!?. Result was {:?}", s)
                },
                DiagServerRx::RecvError(e) => {
                    return Err(e)
                }
            }
        }
    }

    /// Returns true only if a hardware failure has resulted in the ECU
    /// disconnecting from the diagnostic server.
    pub fn is_ecu_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub fn register_waiting_hook<F: FnMut() + 'static>(&mut self, hook: F) {
        self.waiting_hook = Box::new(hook)
    }

    pub fn register_send_complete_hook<F: FnMut() + 'static>(&mut self, hook: F) {
        self.on_send_complete_hook = Box::new(hook)
    }

    pub fn get_current_diag_mode(&self) -> Option<DiagSessionMode> {
        self.current_diag_mode.read().unwrap().clone()
    }

    fn clear_rx_queue(&self) {
        while self.receiver.try_recv().is_ok(){}
    }
}

impl Drop for DynamicDiagSession {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed)
    }
}

fn send_recv_ecu_req<P, NRC>(
    protocol: &P,
    tx_addr: u32,
    ext_id: Option<u8>,
    payload: &[u8], // If empty, we are only reading
    needs_response: bool,
    tx_resp: Option<&mut Sender<DiagServerRx>>,
    basic_opts: DiagServerBasicOptions,
    cooldown: u32,
    channel: &mut Box<dyn IsoTPChannel>,
    connect_state: &AtomicBool
) -> DiagServerRx
where
    P: DiagProtocol<NRC>,
    NRC: EcuNRC {
        // Send the request, and transmit the send state!
        let mut res: ChannelResult<()> = Ok(());
        if !payload.is_empty() { // We need to write some bytes
            log::debug!("Sending req to ECU: {:02X?}", payload);
            res = channel.clear_tx_buffer()
                .and_then(|_| channel.clear_rx_buffer())
                .and_then(|_|
                    channel.write_bytes(tx_addr, ext_id, &payload, basic_opts.timeout_cfg.write_timeout_ms)
                )
        }
        match res {
            Ok(_) => {
                if needs_response {
                    log::debug!("Sending OK, awaiting response from ECU");
                    // Notify sending has completed, we will now poll for the ECUs response!
                    if let Some(s) = &tx_resp {
                        s.send(DiagServerRx::SendState(Ok(()))).unwrap();
                    }
                    // Now poll for the ECU's response
                    match channel.read_bytes(basic_opts.timeout_cfg.read_timeout_ms) {
                        Err(e) => {
                            log::error!("Error reading from channel: {:02X?}", payload);
                            connect_state.store(false, Ordering::Relaxed);
                            // Final error
                            return DiagServerRx::RecvError(e.into())
                        },
                        Ok(bytes) => {
                            log::debug!("ECU Response: {:02X?}", bytes);
                            let parsed_response = P::process_ecu_response(&bytes);
                            connect_state.store(true, Ordering::Relaxed);
                            return match parsed_response {
                                Ok(pos_result) => {
                                    log::debug!("ECU Response OK!");
                                    DiagServerRx::EcuResponse(pos_result)
                                },
                                Err((code, nrc_data)) => {
                                    if nrc_data.is_ecu_busy() {
                                        // ECU waiting, so poll again for the response
                                        // to do that, call this function again with no payload
                                        log::debug!("ECU is busy, awaiting response");
                                        return send_recv_ecu_req(protocol, tx_addr, ext_id, &[], needs_response, tx_resp, basic_opts, cooldown, channel, connect_state)
                                    } else if nrc_data.is_repeat_request() {
                                        // ECU wants us to ask again, so we wait a little bit, then call ourselves again
                                        log::debug!("ECU has asked for a repeat of the request");
                                        std::thread::sleep(Duration::from_millis(cooldown.into()));
                                        return send_recv_ecu_req(protocol, tx_addr, ext_id, payload, needs_response, tx_resp, basic_opts, cooldown, channel, connect_state)
                                    } else {
                                        // Unhandled NRC
                                        log::warn!("ECU Negative response {:02X?}", code);
                                        DiagServerRx::EcuError {b: code, desc: nrc_data.desc()}
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Final state. We are done!
                    log::debug!("No need to poll ECU response");
                    connect_state.store(true, Ordering::Relaxed);
                    return DiagServerRx::SendState(Ok(()))
                }
            },
            Err(e) => {
                log::error!("Channel send error: {}", e);
                // Final error here at send state :(
                connect_state.store(false, Ordering::Relaxed);
                return DiagServerRx::SendState(Err(e.into()));
            },
        }
}
