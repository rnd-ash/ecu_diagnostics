//! Module for OBD (ISO-9141)

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Instant,
};

use crate::{
    channel::{IsoTPChannel, IsoTPSettings},
    helpers, BaseServerPayload, BaseServerSettings, DiagError, DiagServerResult, DiagnosticServer,
    ServerEvent, ServerEventHandler,
};

mod data_pids;
mod enumerations;
mod service01;
mod service09;
mod units;

// Exports
use crate::dtc::{DTCFormatType, DTCStatus, DTC};
pub use data_pids::*;
pub use enumerations::*;
pub use service01::*;
pub use service09::*;
pub use units::*;

// OBD2 does not have a 'session type' like KWP or UDS,
// so create a dummy marker just to satisfy the <VoidHandler> trait
struct VoidSessionType;

/// Function to decode PID support response from ECU
pub(crate) fn decode_pid_response(x: &[u8]) -> Vec<bool> {
    let mut resp: Vec<bool> = Vec::new();
    for b in x {
        let mut mask: u8 = 0b10000000;
        for _ in 0..8 {
            resp.push(b & mask != 0x00);
            mask >>= 1;
        }
    }
    resp
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
/// OBD2 Service IDs
pub enum OBD2Command {
    /// Service 01 - Show current data
    Service01,
    /// Service 02 - Show freeze frame data
    Service02,
    /// Service 03 - Show stored DTCs
    Service03,
    /// Service 04 - Clear stored DTCs
    Service04,
    /// Test results, O2 sensor monitoring (non CAN)
    Service05,
    /// Test results, O2 sensor monitoring (CAN)
    Service06,
    /// Show pending DTCs
    Service07,
    /// Control operation of on-board components
    Service08,
    /// Service 09 - Request vehicle information
    Service09,
    /// Service 0A - Read permanent DTCs
    Service0A,
    /// Custom OBD service. Not 0x10+ is either KWP or UDS!
    Custom(u8),
}

impl From<u8> for OBD2Command {
    fn from(sid: u8) -> Self {
        match sid {
            0x01 => Self::Service01,
            0x02 => Self::Service02,
            0x03 => Self::Service03,
            0x04 => Self::Service04,
            0x05 => Self::Service05,
            0x06 => Self::Service06,
            0x07 => Self::Service07,
            0x08 => Self::Service08,
            0x09 => Self::Service09,
            0x0A => Self::Service0A,
            _ => Self::Custom(sid),
        }
    }
}

impl From<OBD2Command> for u8 {
    fn from(cmd: OBD2Command) -> Self {
        match cmd {
            OBD2Command::Service01 => 0x01,
            OBD2Command::Service02 => 0x02,
            OBD2Command::Service03 => 0x03,
            OBD2Command::Service04 => 0x04,
            OBD2Command::Service05 => 0x05,
            OBD2Command::Service06 => 0x06,
            OBD2Command::Service07 => 0x07,
            OBD2Command::Service08 => 0x08,
            OBD2Command::Service09 => 0x09,
            OBD2Command::Service0A => 0x0A,
            OBD2Command::Custom(x) => x,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
/// Wrapper round OBD2 protocol NRC codes
pub enum OBD2Error {
    /// ECU general reject
    GeneralReject,
    /// Service is not supported in active session.
    /// This is a weird error for OBD as OBD only has
    /// one session mode
    ServiceNotSupportedInActiveSession,
    /// Request message format was incorrect
    FormatIncorrect,
    /// Requested data was out of range
    OutOfRange,
    /// ECU is busy, repeat the request
    BusyRepeatRequest,
    /// ECU is busy, but will respond to the original request shortly
    BusyResponsePending,
    /// Conditions are not correct to execute the request
    ConditionsNotCorrect,
    /// Out of order request in a sequence of request
    RequestSequenceError,
    /// Security access is denied
    SecurityAccessDenied,
    /// Invalid security key
    InvalidKey,
    /// Exceeded the maximum number of attempts at authentication
    ExceedAttempts,
    /// OBD NRC. This can mean different things per OEM
    Custom(u8),
}

impl From<u8> for OBD2Error {
    fn from(p: u8) -> Self {
        match p {
            0x10 => Self::GeneralReject,
            0x11 | 0x12 | 0x7E | 0x7F => Self::ServiceNotSupportedInActiveSession,
            0x13 => Self::FormatIncorrect,
            0x31 => Self::OutOfRange,
            0x21 => Self::BusyRepeatRequest,
            0x78 => Self::BusyResponsePending,
            0x22 => Self::ConditionsNotCorrect,
            0x24 => Self::RequestSequenceError,
            0x33 => Self::SecurityAccessDenied,
            0x35 => Self::InvalidKey,
            0x36 => Self::ExceedAttempts,
            x => Self::Custom(x),
        }
    }
}

#[derive(Clone, Debug)]
/// Kwp2000 message payload
pub struct OBD2Cmd(Vec<u8>);

impl OBD2Cmd {
    /// Creates a new OBD2 Payload
    pub fn new(sid: OBD2Command, args: &[u8]) -> Self {
        let mut b: Vec<u8> = Vec::with_capacity(args.len() + 1);
        b.push(u8::from(sid));
        b.extend_from_slice(args);
        Self(b)
    }

    pub(crate) fn from_raw(s: &[u8]) -> Self {
        Self(s.to_vec())
    }

    /// Returns the OBD2 Service ID of the command
    pub fn get_obd_sid(&self) -> OBD2Command {
        self.0[0].into()
    }
}

impl BaseServerPayload for OBD2Cmd {
    fn get_payload(&self) -> &[u8] {
        &self.0[1..]
    }

    fn get_sid_byte(&self) -> u8 {
        self.0[0]
    }

    fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    fn requires_response(&self) -> bool {
        true
    }
}

/// Base handler for OBD2
#[derive(Debug, Copy, Clone)]
pub struct OBD2VoidHandler;

impl ServerEventHandler<VoidSessionType> for OBD2VoidHandler {
    #[inline(always)]
    fn on_event(&mut self, _e: ServerEvent<VoidSessionType>) {}
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
/// OBD2 server options
pub struct Obd2ServerOptions {
    /// ECU Send ID
    pub send_id: u32,
    /// ECU Receive ID
    pub recv_id: u32,
    /// Read timeout in ms
    pub read_timeout_ms: u32,
    /// Write timeout in ms
    pub write_timeout_ms: u32,
}

impl BaseServerSettings for Obd2ServerOptions {
    fn get_write_timeout_ms(&self) -> u32 {
        self.write_timeout_ms
    }

    fn get_read_timeout_ms(&self) -> u32 {
        self.read_timeout_ms
    }
}

fn lookup_obd_nrc(x: u8) -> String {
    format!("{:?}", OBD2Error::from(x))
}

#[derive(Debug)]
/// OBD2 Diagnostic server
pub struct OBD2DiagnosticServer {
    server_running: Arc<AtomicBool>,
    settings: Obd2ServerOptions,
    tx: mpsc::Sender<OBD2Cmd>,
    rx: mpsc::Receiver<DiagServerResult<Vec<u8>>>,
    repeat_count: u32,
    repeat_interval: std::time::Duration,
}

impl OBD2DiagnosticServer {
    /// Creates a new OBD2 over an ISO-TP connection with the ECU
    ///
    /// On startup, this server will configure the channel with the necessary settings provided in both
    /// settings and channel_cfg
    ///
    /// ## Parameters
    /// * settings - OBD2 Server settings
    /// * channel - ISO-TP communication channel with the ECU
    /// * channel_cfg - The settings to use for the ISO-TP channel
    pub fn new_over_iso_tp<C>(
        settings: Obd2ServerOptions,
        mut server_channel: C,
        channel_cfg: IsoTPSettings,
    ) -> DiagServerResult<Self>
    where
        C: IsoTPChannel + 'static,
    {
        server_channel.set_iso_tp_cfg(channel_cfg)?;
        server_channel.set_ids(settings.send_id, settings.recv_id)?;
        server_channel.open()?;

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_t = is_running.clone();

        let (tx_cmd, rx_cmd) = mpsc::channel::<OBD2Cmd>();
        let (tx_res, rx_res) = mpsc::channel::<DiagServerResult<Vec<u8>>>();

        std::thread::spawn(move || {
            log::debug!("server start");
            loop {
                if !is_running_t.load(Ordering::Relaxed) {
                    log::debug!("server exit");
                    break;
                }

                if let Ok(cmd) = rx_cmd.try_recv() {
                    // We have an incoming command
                    log::debug!(
                        "Incoming request from tester. Sending {:02X?} to ECU",
                        cmd
                    );
                    let res = helpers::perform_cmd(
                        settings.send_id,
                        &cmd,
                        &settings,
                        &mut server_channel,
                        0x21,
                        lookup_obd_nrc,
                    );
                    //event_handler.on_event(&res);
                    if tx_res.send(res).is_err() {
                        // Terminate! Something has gone wrong and data can no longer be sent to client
                        is_running_t.store(false, Ordering::Relaxed);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        Ok(Self {
            server_running: is_running,
            tx: tx_cmd,
            rx: rx_res,
            settings,
            repeat_count: 3,
            repeat_interval: std::time::Duration::from_millis(1000),
        })
    }

    /// Returns the current settings used by the OBD2 Server
    pub fn get_settings(&self) -> Obd2ServerOptions {
        self.settings
    }

    /// Internal command for sending KWP2000 payload to the ECU
    fn exec_command(&mut self, cmd: OBD2Cmd) -> DiagServerResult<Vec<u8>> {
        match self.tx.send(cmd) {
            Ok(_) => self.rx.recv().unwrap_or(Err(DiagError::ServerNotRunning)),
            Err(_) => Err(DiagError::ServerNotRunning), // Server must have crashed!
        }
    }

    /// Attempts to clear stored DTCs on the ECU using Service 04
    pub fn clear_dtcs(&mut self) -> DiagServerResult<()> {
        self.exec_command(OBD2Cmd::new(OBD2Command::Service04, &[]))
            .map(|_| ())
    }

    /// Attempts to read all stored DTCs on the ECU, using a combination of
    /// Services 03, 07 and 0A (Stored, Pending and Permanent)
    pub fn read_dtcs(&mut self) -> DiagServerResult<Vec<DTC>> {
        let mut res: Vec<DTC> = Vec::new();

        if let Ok(mut resp) = self.exec_command(OBD2Cmd::new(OBD2Command::Service03, &[])) {
            resp.drain(0..2);
            if resp.len() % 2 != 0 {
                return Err(DiagError::InvalidResponseLength);
            }
            for dtc_bytes in resp.chunks(2) {
                res.push(DTC {
                    format: DTCFormatType::Iso15031_6,
                    raw: ((dtc_bytes[0] as u32) << 8) | dtc_bytes[1] as u32,
                    status: DTCStatus::Pending,
                    mil_on: false,
                    readiness_flag: false,
                })
            }
        }

        if let Ok(mut resp) = self.exec_command(OBD2Cmd::new(OBD2Command::Service07, &[])) {
            resp.drain(0..2);
            if resp.len() % 2 != 0 {
                return Err(DiagError::InvalidResponseLength);
            }
            for dtc_bytes in resp.chunks(2) {
                if let Some(existing_dtc) = res
                    .iter_mut()
                    .find(|x| x.raw == ((dtc_bytes[0] as u32) << 8) | dtc_bytes[1] as u32)
                {
                    existing_dtc.status = DTCStatus::Stored;
                    existing_dtc.mil_on = true;
                } else {
                    res.push(DTC {
                        format: DTCFormatType::Iso15031_6,
                        raw: ((dtc_bytes[0] as u32) << 8) | dtc_bytes[1] as u32,
                        status: DTCStatus::Stored,
                        mil_on: true,
                        readiness_flag: false,
                    })
                }
            }
        }

        if let Ok(mut resp) = self.exec_command(OBD2Cmd::new(OBD2Command::Service0A, &[])) {
            resp.drain(0..2);
            if resp.len() % 2 != 0 {
                return Err(DiagError::InvalidResponseLength);
            }
            for dtc_bytes in resp.chunks(2) {
                if let Some(existing_dtc) = res
                    .iter_mut()
                    .find(|x| x.raw == ((dtc_bytes[0] as u32) << 8) | dtc_bytes[1] as u32)
                {
                    existing_dtc.status = DTCStatus::Permanent;
                    existing_dtc.mil_on = true;
                } else {
                    res.push(DTC {
                        format: DTCFormatType::Iso15031_6,
                        raw: ((dtc_bytes[0] as u32) << 8) | dtc_bytes[1] as u32,
                        status: DTCStatus::Permanent,
                        mil_on: true,
                        readiness_flag: false,
                    })
                }
            }
        }
        Ok(res)
    }
}

impl DiagnosticServer<OBD2Command> for OBD2DiagnosticServer {
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
        sid: OBD2Command,
        args: &[u8],
    ) -> DiagServerResult<Vec<u8>> {
        let cmd = OBD2Cmd::new(sid, args);

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
    fn execute_command(&mut self, sid: OBD2Command, args: &[u8]) -> DiagServerResult<()> {
        let cmd = OBD2Cmd::new(sid, args);
        self.exec_command(cmd).map(|_| ())
    }

    /// Sends an arbitrary byte array to the ECU.
    /// NOTE: On OBD2, this function will block as we HAVE to poll for the ECU
    /// response on OBD2!
    fn send_byte_array(&mut self, arr: &[u8]) -> DiagServerResult<()> {
        self.send_byte_array_with_response(arr).map(|_| ())
    }

    /// Sends an arbitrary byte array to the ECU, and polls for the ECU's response
    fn send_byte_array_with_response(&mut self, arr: &[u8]) -> DiagServerResult<Vec<u8>> {
        let cmd = OBD2Cmd::from_raw(arr);
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

    /// Returns true if the internal OBD2 Server is running
    fn is_server_running(&self) -> bool {
        self.server_running.load(Ordering::Relaxed)
    }
}

/// Returns the OBD2 error from a given error code
pub fn get_description_of_ecu_error(error: u8) -> OBD2Error {
    error.into()
}

impl Drop for OBD2DiagnosticServer {
    fn drop(&mut self) {
        self.server_running.store(false, Ordering::Relaxed); // Stop server
    }
}

#[cfg(test)]
pub mod obd_test {
    use crate::obd2::decode_pid_response;

    #[test]
    pub fn test_pid_support_decoding() {
        let input = vec![0xBE, 0x1F, 0xA8, 0x13];
        let output = vec![
            true, // B
            false, true, true, true, // E
            true, true, false, false, // 1
            false, false, true, true, // F
            true, true, true, true, // A
            false, true, false, true, // 8
            false, false, false, false, // 1
            false, false, true, false, // 3
            false, true, true,
        ];
        assert_eq!(decode_pid_response(&input), output);
    }
}
