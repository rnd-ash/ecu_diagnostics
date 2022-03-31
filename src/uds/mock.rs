use socketcan_isotp::IsoTpOptions;

use crate::{hardware::simulation::SimulationIsoTpChannel, channel::IsoTPSettings, ServerEventHandler};

use super::{UdsDiagnosticServer, UdsServerOptions, UdsVoidHandler, UDSSessionType};

///! Mocking framework for UDS (For unit testing)

#[derive(Debug)]
pub struct MockUdsServer {
    channel: SimulationIsoTpChannel,
    pub server: UdsDiagnosticServer
}

#[derive(Debug)]
pub struct UdsMockLogger{}

impl ServerEventHandler<UDSSessionType> for UdsMockLogger {
    fn on_event(&mut self, e: crate::ServerEvent<UDSSessionType>) {
        match e {
            crate::ServerEvent::CriticalError { desc } => todo!(),
            crate::ServerEvent::ServerStart => {},
            crate::ServerEvent::ServerExit => {},
            crate::ServerEvent::DiagModeChange { old, new } => {},
            crate::ServerEvent::Request(r) => println!("Out -> {:02X?}", r),
            crate::ServerEvent::Response(r) => println!("In <- {:02X?}", r),
            crate::ServerEvent::TesterPresentError(_) => {},
            crate::ServerEvent::InterfaceCloseOnExitError(_) => {},
        }
    }
}

impl MockUdsServer {
    pub fn new() -> Self {
        let channel = SimulationIsoTpChannel::new();

        let server_options = UdsServerOptions {
            send_id: 0x07E0,
            recv_id: 0x07E8,
            read_timeout_ms: 100,
            write_timeout_ms: 100,
            global_tp_id: 0x00,
            tester_present_interval_ms: 2000,
            tester_present_require_response: true,
        };

        let isotp_settings = IsoTPSettings {
            block_size: 8,
            st_min: 20,
            extended_addressing: false,
            pad_frame: true,
            can_speed: 500_000,
            can_use_ext_addr: false,
        };

        let server = UdsDiagnosticServer::new_over_iso_tp(server_options, channel.clone(), isotp_settings, UdsMockLogger{}).unwrap();
        
        Self {
            channel,
            server
        }
    }

    pub fn add_response(&mut self, req: &[u8], resp: &[u8]) {
        self.channel.add_response(req, resp)
    }

    pub fn start_test(&mut self) {
        // Pre-load all necessary payloads for UDS

        // Tester present
        self.channel.add_response(&[0x3E, 0x00], &[0x7E, 0x00]);

        // Session modes
        self.channel.add_response(&[0x10, 0x01], &[0x50, 0x01]); // Default mode
        self.channel.add_response(&[0x10, 0x03], &[0x50, 0x03]); // Extended mode
    }

    pub fn end_test(&mut self) {
        self.channel.clear_map();
    }
}