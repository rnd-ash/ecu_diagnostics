use std::{intrinsics::transmute, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc}, time::{Duration, Instant}};

use crate::{BaseChannel, DiagError, DiagServerResult, IsoTPChannel, IsoTPSettings};


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UDSCommand {
    DiagnosticSessionControl = 0x10,
    ECUReset = 0x11,
    SecurityAccess = 0x27,
    CommunicationControl = 0x28,
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
    RequestTransferExit = 0x37
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UDSSessionType {
    Default,
    Programming,
    Extended,
    SafetySystem,
    Other(u8)
}

impl Into<u8> for UDSSessionType {
    fn into(self) -> u8 {
        match &self {
            UDSSessionType::Default => 0x01,
            UDSSessionType::Programming => 0x02,
            UDSSessionType::Extended => 0x03,
            UDSSessionType::SafetySystem => 0x04,
            UDSSessionType::Other(x) => *x,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UDSError {
    GeneralReject,
    ServiceNotSupported,
    SubFunctionNotSupported,
    IncorrectMessageLengthOrInvalidFormat,
    ResponseTooLong,
    BusyRepeatRequest,
    ConditionsNotCorrect,
    RequestSequenceError,
    NoResponseFromSubnetComponent,
    FailurePreventsExecutionOfRequestedAction,
    RequestOutOfRange,
    SecurityAccessDenied,
    InvalidKey,
    ExceedNumberOfAttempts,
    RequiredTimeDelayNotExpired,
    UploadDownloadNotAccepted,
    TransferDataSuspended,
    GeneralProgrammingFailure,
    WrongBlockSequenceCounter,
    RequestCorrectlyReceivedResponsePending,
    SubFunctionNotSupportedInActiveSession,
    ServiceNotSupportedInActiveSession,
    RpmTooHigh,
    RpmTooLow,
    EngineIsRunning,
    EngineIsNotRunning,
    EngineRunTimeTooLow,
    TemperatureTooHigh,
    TemperatureTooLow,
    VehicleSpeedTooHigh,
    VehicleSpeedTooLow,
    ThrottleTooHigh,
    ThrottleTooLow,
    TransmissionRangeNotInNeutral,
    TransmissionRangeNotInGear,
    BrakeSwitchNotClosed,
    ShifterLeverNotInPark,
    TorqueConverterClutchLocked,
    VoltageTooHigh,
    VoltageTooLow,
    ReserverdForSpecificConditionsNotCorrect,
    ReservedByExtendedDataLinkSecurityDocumentation,
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
pub struct UdsServerOptions {
    pub baud: u32,
    pub send_id: u32,
    pub recv_id: u32,
    pub read_timeout_ms: u32,
    pub write_timeout_ms: u32,
    pub global_tp_id: Option<u32>,
    pub tester_present_interval_ms: u32,
    pub server_refresh_interval_ms: u32,
    pub tester_present_require_response: bool
}


#[derive(Debug, Clone)]
struct UdsCmd {
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
            response_required: need_response
        }
    }

    pub fn get_sid(&self) -> UDSCommand {
        unsafe { transmute(self.bytes[0]) } // This unsafe operation will always succeed!
    }
}

#[derive(Debug)]
pub struct UdsDiagnosticServer{
    server_running: Arc<AtomicBool>,
    settings: UdsServerOptions,
    tx: mpsc::Sender<UdsCmd>,
    rx: mpsc::Receiver<DiagServerResult<Vec<u8>>>
}

impl UdsDiagnosticServer {
    pub fn new_over_iso_tp(settings: UdsServerOptions, channel: Box<dyn IsoTPChannel>, channel_cfg: IsoTPSettings) -> DiagServerResult<Self> {

        let mut server_channel = channel.clone();

        server_channel.set_baud(settings.baud)?;
        server_channel.set_ids(settings.send_id, settings.recv_id, settings.global_tp_id)?;
        server_channel.configure_iso_tp(channel_cfg)?;

        let is_running = Arc::new(AtomicBool::new(true));
        let is_running_t = is_running.clone();

        let (tx_cmd, rx_cmd) = mpsc::channel::<UdsCmd>();
        let (tx_res, rx_res) = mpsc::channel::<DiagServerResult<Vec<u8>>>();

        let server_opts = settings.clone();

        std::thread::spawn(move|| {
            
            fn check_pos_response_id(sid: u8, resp: Vec<u8>) -> DiagServerResult<Vec<u8>> {
                if resp[0] != sid + 0x40 {
                    Err(DiagError::WrongMessage)
                } else {
                    Ok(resp)
                }
            }

            fn perform_cmd(cmd: UdsCmd, settings: &UdsServerOptions, channel: &mut Box<dyn BaseChannel>) -> DiagServerResult<Vec<u8>> {
                // Clear IO buffers
                channel.clear_rx_buffer()?;
                channel.clear_tx_buffer()?;
                let target = cmd.bytes[0];
                if !cmd.response_required {
                    // Just send the data and return an empty response
                    channel.write_bytes(&cmd.bytes, settings.write_timeout_ms)?;
                    return Ok(Vec::new())
                }
                let res = channel.read_write_bytes(&cmd.bytes, settings.write_timeout_ms, settings.read_timeout_ms)?;
                if res.is_empty() {
                    return Err(DiagError::EmptyResponse)
                }
                if res[0] == 0x7F {
                    if UDSError::from(res[1]) == UDSError::RequestCorrectlyReceivedResponsePending {
                        // Wait a bit longer for the ECU response
                        let timestamp = Instant::now();
                        while timestamp.elapsed() <= Duration::from_millis(1000) {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            if let Ok(res2) = channel.read_bytes(settings.read_timeout_ms) {
                                if res2.is_empty() {
                                    return Err(DiagError::EmptyResponse)
                                }
                                if res2[0] == 0x7F {
                                    // Still an error. Give up
                                    return Err(super::DiagError::ECUError(res2[1].into()))
                                } else {
                                    // Response OK! Set last tester time so we don't flood the ECU too quickly
                                    return check_pos_response_id(target, res2)
                                }
                            }
                        }
                        // No response! Return the last error
                        return Err(super::DiagError::ECUError(res[1].into()))
                    } else {
                        // Other error! - Return that error
                        return Err(super::DiagError::ECUError(res[1].into()))
                    }
                }
                check_pos_response_id(target, res) // ECU Response OK!
            }

            let mut send_tester_present = false;
            let mut last_tester_present_time: Instant = Instant::now();
            
            let mut base_channel = server_channel.clone_base();
            loop {
                if is_running_t.load(Ordering::Relaxed) == false {
                    break
                }

                if let Ok(cmd) = rx_cmd.try_recv() {
                    // We have an incoming command
                    if cmd.get_sid() == UDSCommand::DiagnosticSessionControl {
                        // Session change! Handle this differently
                        match perform_cmd(cmd.clone(), &settings, &mut base_channel) {
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
                                    is_running_t.store(false, Ordering::Relaxed)
                                }
                            },
                            Err(e) => {
                                if let Err(_) = tx_res.send(Err(e)) {
                                    // Terminate! Something has gone wrong and data can no longer be sent to client
                                    is_running_t.store(false, Ordering::Relaxed)
                                }
                            }
                        }
                    } else {
                        // Generic command just perform it
                        if let Err(_) = tx_res.send(perform_cmd(cmd, &settings, &mut base_channel)) {
                            // Terminate! Something has gone wrong and data can no longer be sent to client
                            is_running_t.store(false, Ordering::Relaxed)
                        }
                    }
                }

                // Deal with tester present
                if send_tester_present && last_tester_present_time.elapsed().as_millis() as u32 >= settings.tester_present_interval_ms {
                    // Send tester present message
                    let cmd = UdsCmd::new(UDSCommand::TesterPresent, &[0x00], true);
                    perform_cmd(cmd, &settings, &mut base_channel);
                    last_tester_present_time = Instant::now();
                    
                }

                std::thread::sleep(std::time::Duration::from_millis(server_opts.server_refresh_interval_ms as u64));
            }
        });

        Ok(Self {
            server_running: is_running,
            tx: tx_cmd,
            rx: rx_res,
            settings
        })
    }

    pub fn is_server_running(&self) -> bool {
        self.server_running.load(Ordering::Relaxed)
    }

    pub fn get_settings(&self) -> UdsServerOptions {
        self.settings
    }

    pub fn executeCommandWithResponse(&mut self, sid: UDSCommand, args: &[u8]) -> DiagServerResult<Vec<u8>> {
        let cmd = UdsCmd::new(sid, args, true);
        self.execCommand(cmd)
    }

    pub fn executeCommand(&mut self, sid: UDSCommand, args: &[u8]) -> DiagServerResult<()> {
        let cmd = UdsCmd::new(sid, args, false);
        self.execCommand(cmd).map(|_| ())
    }

    fn execCommand(&mut self, cmd: UdsCmd) -> DiagServerResult<Vec<u8>> {
        match self.tx.send(cmd) {
            Ok(_) => {
                return self.rx.recv().unwrap_or(Err(DiagError::ServerNotRunning))
            },
            Err(_) => return Err(DiagError::ServerNotRunning) // Server must have crashed!
        }
    }

}


unsafe impl Sync for UdsDiagnosticServer{}
unsafe impl Send for UdsDiagnosticServer{}


#[cfg(test)]
pub mod UdsServerTest {
    use super::*;


    #[derive(Clone)]
    pub struct UdsSimEcu<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> {
        on_data_callback: T,
        out_buffer: Vec<Vec<u8>>
    }
    unsafe impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> Send for UdsSimEcu<T> {}
    unsafe impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> Sync for UdsSimEcu<T> {}
    

    impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> UdsSimEcu<T> {
        pub fn new(on_data_callback: T) -> Self {
            Self { on_data_callback, out_buffer: Vec::new() }
        }

        pub fn set_callback(&mut self, on_data_callback: T) {
            self.on_data_callback = on_data_callback
        }
    }

    impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> IsoTPChannel for UdsSimEcu<T> {
        fn configure_iso_tp(&mut self, cfg: IsoTPSettings) -> DiagServerResult<()> {
            println!("IsoTPChannel: configure_iso_tp Called. BS: {}, ST-MIN: {}", cfg.block_size, cfg.st_min);
            Ok(())
        }

        fn clone_isotp(&self) -> Box<dyn IsoTPChannel> {
            println!("IsoTPChannel: clone_isotp Called");
            Box::new(self.clone())
        }

        fn into_base(&self) -> Box<dyn BaseChannel> {
            println!("IsoTPChannel: into_base Called");
            Box::new(self.clone())
        }
    }

    impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> BaseChannel for UdsSimEcu<T> {
        fn clone_base(&self) -> Box<dyn BaseChannel> {
            println!("BaseChannel: into_base Called");
            Box::new(self.clone())
        }

        fn set_baud(&mut self, baud: u32) -> DiagServerResult<()> {
            println!("BaseChannel: set_baud Called. Baud: {} bps", baud);
            Ok(())
        }

        fn set_ids(&mut self, send: u32, recv: u32, global_tp_id: Option<u32>) -> DiagServerResult<()> {
            println!("BaseChannel: set_ids Called. send: {}, recv: {}, global_tp_id: {:?}", send, recv, global_tp_id);
            Ok(())
        }

        fn read_bytes(&mut self, timeout_ms: u32) -> DiagServerResult<Vec<u8>> {
            println!("BaseChannel: read_bytes Called. timeout_ms: {}", timeout_ms);
            if self.out_buffer.is_empty() {
                println!("-- NOTHING TO SEND");
                Err(DiagError::Timeout)
            } else {
                let send = self.out_buffer[0].clone();
                println!("-- Sending {:02X?} back to diag server", &send);
                self.out_buffer.drain(0..1);
                Ok(send)
            }
        }

        fn write_bytes(&mut self, buffer: &[u8], timeout_ms: u32) -> DiagServerResult<()> {
            println!("BaseChannel: write_bytes Called. Tx: {:02X?}, timeout_ms: {}", buffer, timeout_ms);
            if let Some(sim_resp) = (self.on_data_callback)(buffer) {
                self.out_buffer.push(sim_resp);
            }
            Ok(())
        }

        fn clear_rx_buffer(&mut self) -> DiagServerResult<()> {
            self.out_buffer = Vec::new();
            Ok(())
        }

        fn clear_tx_buffer(&mut self) -> DiagServerResult<()> {
            Ok(())
        }
    }


    #[test]
    pub fn test_send_uds_cmd() {
        fn callback(buf: &[u8]) -> Option<Vec<u8>> {
            if buf[0] == 0x10 { // Start ID
                return Some(vec![0x50, buf[1]])
            } else {
                None
            }
        }

        let sim_ecu = UdsSimEcu::new(callback);


        let settings = UdsServerOptions {
            baud: 500000,
            send_id: 0x07E0,
            recv_id: 0x07E8,
            read_timeout_ms: 1000,
            write_timeout_ms: 1000,
            global_tp_id: None,
            tester_present_interval_ms: 2000,
            server_refresh_interval_ms: 10,
            tester_present_require_response: true,
        };

        let mut server = UdsDiagnosticServer::new_over_iso_tp(
            settings, 
            Box::new(sim_ecu), 
            IsoTPSettings {
                block_size: 8,
                st_min: 20,
            }
        ).unwrap();

        server.executeCommandWithResponse(UDSCommand::DiagnosticSessionControl, &[0x10]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5000));

    }

}