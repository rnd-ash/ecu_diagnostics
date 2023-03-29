//! Dynamic diagnostic session helper
//!

use std::{
    sync::{Arc, Mutex, RwLock, mpsc}, time::Instant,
};

use crate::{
    channel::IsoTPSettings,
    hardware::Hardware,
    DiagError, DiagServerResult, helpers
};

/// Dynamic diagnostic session
///
/// This is used if a target ECU has an unknown diagnostic protocol.
///
/// This also contains some useful wrappers for basic functions such as
/// reading and clearing error codes.
#[derive(Debug)]
pub struct DynamicDiagSession<P> where P : DiagProtocol {
    current_session_mode: Arc<RwLock<DiagSessionMode>>,
    protocol: P,
    sender: mpsc::Sender<DiagTxPayload>,
    receiver: mpsc::Receiver<DiagServerResult<DiagServerRx>>
}

#[derive(Debug, Copy, Clone)]
pub enum DiagServerRx {
    EcuResponse(Vec<u8>),
    EcuError(u8),
    EcuWaiting,
    SendState(DiagServerResult<()>)
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
/// Basic diagnostic server options
pub struct DiagServerBasicOptions {
    /// ECU Send ID
    pub send_id: u32,
    /// ECU Receive ID
    pub recv_id: u32,
    /// Read timeout in ms
    pub read_timeout_ms: u32,
    /// Write timeout in ms
    pub write_timeout_ms: u32
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
    /// Cooldown period in MS after receiving a response from an ECU before sending a request.
    /// This is useful for some slower ECUs
    pub command_cooldown_ms: u128
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagSessionMode {
    /// Session mode ID
    id: u8,
    /// Tester present required?
    tp_require: bool,
    /// Alias for its name (For logging only)
    name: &'static str
}

pub trait DiagSID: From<u8> {

}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagAction {
    SetSessionMode(DiagSessionMode),
    Other { sid: dyn DiagSID, data: Vec<u8> }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiagTxPayload {
    pub action: DiagAction,
    pub response_require: bool
}

pub trait DiagProtocol {
    /// Returns the alias to the ECU 'default' diagnostic session mode
    /// Returns None if there is no session type control in the protocol
    /// (For example basic OBD2)
    fn get_basic_session_mode() -> Option<DiagSessionMode>;
    /// Name of the diagnostic protocol
    fn get_protocol_name() -> &'static str;
    /// Process a byte array into a command
    fn process_req_payload(payload: &[u8]) -> DiagAction;
    /// Generate the tester present message (If required)
    fn create_tp_msg(response_required: bool) -> DiagAction;
    /// Processes the ECU response, and checks to see if it is a positive or negative response
    fn process_ecu_response(r: &[u8]) -> DiagServerRx;
}

impl<P> DynamicDiagSession<P> where P: DiagProtocol {
    /// Creates a new dynamic session.
    /// This will first try with KWP2000, then if that fails,
    /// will try with UDS. If both server creations fail,
    /// then the last error will be returned.
    ///
    /// NOTE: In order to test if the ECU supports the protocol,
    /// the ECU will be put into extended diagnostic session briefly to test
    /// if it supports the tested diagnostic protocol.
    #[allow(unused_must_use, unused_assignments)]
    pub fn new_over_iso_tp<C>(
        hw_device: Arc<Mutex<C>>,
        channel_cfg: IsoTPSettings,
        basic_opts: DiagServerBasicOptions,
        advanced_opts: Option<DiagServerAdvancedOptions>
    ) -> DiagServerResult<Self>
    where
        C: Hardware + 'static
    {
        let mut last_err: Option<DiagError>; // Setting up last recorded error

        // Create iso tp channel using provided HW interface. If this fails, we cannot setup KWP or UDS session!
        let mut iso_tp_channel = Hardware::create_iso_tp_channel(hw_device.clone())?;
        let requested_session_mode = P::get_basic_session_mode();
        let mut current_session_mode = P::get_basic_session_mode();
        if requested_session_mode.is_none() && advanced_opts.is_some() {
            log::warn!("Session mode is None but advanced opts was specified. Ignoring advanced opts");
        }
        let session_control = current_session_mode.is_some() && advanced_opts.is_some();
        std::thread::spawn(|| {
            let mut last_tp_time = Instant::now();
            let mut waiting_for_response = false;
            loop {
                // We are waiting for a response from the ECU (ReponsePending)
                


                // Logic for handling session control TP present requests
                if session_control {
                    let c_mode = current_session_mode.unwrap();
                    let advanced_opts = advanced_opts.unwrap();
                    if c_mode.tp_require && last_tp_time.elapsed() >= advanced_opts.tester_present_interval_ms {
                        let tx_payload = P::create_tp_msg(advanced_opts.tester_present_require_response);
                        
                    }
                }
            }
        });
    }

    pub fn send_byte_array(&self, p: &[u8]) -> DiagServerResult<()> {
        self.internal_send_byte_array(p, false)
    }

    fn internal_send_byte_array(&self, p: &[u8], resp_require: bool) -> DiagServerResult<()> {
        let parsed = P::process_req_payload(p);
        self.clear_rx_queue();
        self.sender.send(DiagTxPayload { action: parsed, response_require: false });
        loop {
            if let DiagServerRx::SendState(res) = self.receiver.recv().unwrap() {
                return res
            }
        }
    }

    /// Send bytes to the ECU and await its response
    /// ## Params
    /// * p - Raw byte array to send
    /// * on_ecu_waiting_hook - Callback to call when the ECU responds with ResponsePending. Can be used to update a programs state
    /// such that the user is aware the ECU is just processing the request
    pub fn send_byte_array_with_response<F: FnMut()>(&self, p: &[u8], on_ecu_waiting_hook: Option<F>) -> DiagServerResult<Vec<u8>> {
        self.internal_send_byte_array(p, true)?;
        loop {
            match self.receiver.recv().unwrap() {
                Ok(r) => {
                    match r {
                        DiagServerRx::EcuResponse(r) => {
                            return Ok(r)
                        },
                        DiagServerRx::EcuError(e) => {
                            return Err(DiagError::ECUError { code: e, def: None })
                        },
                        DiagServerRx::EcuWaiting => {
                            if let Some(mut waiting_hook) = on_ecu_waiting_hook {
                                (waiting_hook)()
                            }
                        },
                        DiagServerRx::SendState(s) => {
                            log::error!("Multiple send states received!?. Result was {:?}", s)
                        },
                    }
                },
                Err(e) => return Err(e),
            }
        }
    }

    fn clear_rx_queue(&self) {
        while self.receiver.try_recv().is_ok(){}
    }
    

}
