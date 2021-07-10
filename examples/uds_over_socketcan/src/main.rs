use std::process::exit;

use socketcan_iface::SocketCANInterface;

mod sim_ecu;
mod socketcan_iface;

use ecu_diagnostics::{channel::{BaseChannel, ChannelResult, IsoTPChannel, IsoTPSettings}, uds::{self, UdsDiagnosticServer, UdsServerOptions, UdsVoidHandler}};

fn main() {
    // Vcan device
    let vcan0 = SocketCANInterface::new("vcan0".to_string());

    // Setup UDS server
    let uds_server_settings = UdsServerOptions {
        send_id: 0x07E0,
        recv_id: 0x07E8,
        read_timeout_ms: 1000,
        write_timeout_ms: 1000,
        global_tp_id: None,
        tester_present_interval_ms: 2500,
        tester_present_require_response: true,
    };

    let isotp_cfg = IsoTPSettings {
        block_size: 8,
        st_min: 20,
        extended_addressing: false,
        pad_frame: true,
        can_speed: 500000,
        can_use_ext_addr: false,
    };

    let mut uds_server: UdsDiagnosticServer = 
        match UdsDiagnosticServer::new_over_iso_tp(uds_server_settings, vcan0, isotp_cfg, UdsVoidHandler) {
            Ok(server) => server,
            Err(e) => {
                eprintln!("Error setting up UDS server!: {}", e);
                exit(1)
            }
        };

    uds::diagnostic_session_control::set_extended_mode(&mut uds_server); // Set ECU to extended mode

    
}