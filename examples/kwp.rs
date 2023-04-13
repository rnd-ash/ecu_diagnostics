use std::time::Duration;

use ecu_diagnostics::{hardware::{HardwareScanner, self}, channel::{self, IsoTPSettings}, kwp2000::{Kwp2000Protocol, KwpSessionType}, dynamic_diag::{DiagServerBasicOptions, DiagServerAdvancedOptions, TimeoutConfig}};
use socketcan_isotp::{IsoTpOptions, IsoTpSocket};

extern crate ecu_diagnostics;

fn ecu_waiting_hook() {
    println!("ECU is processing our request");
}

fn tx_ok_hook() {
    println!("Data was sent OK!");
}

fn main() {
    env_logger::init();
    let dev = ecu_diagnostics::hardware::socketcan::SocketCanScanner::new();
    let d = dev.open_device_by_name("can0").unwrap();
    
    let protocol = Kwp2000Protocol::new();

    let mut diag_server = 
    ecu_diagnostics::dynamic_diag::DynamicDiagSession::new_over_iso_tp(
        protocol, 
        d, 
        IsoTPSettings { // ISO-TP layer settings
            block_size: 8,
            st_min: 20,
            extended_addresses: None,
            pad_frame: true,
            can_speed: 500_000,
            can_use_ext_addr: false,
        }, 
        DiagServerBasicOptions { // Basic server options
            send_id: 0x07E1,
            recv_id: 0x07E9,
            timeout_cfg: TimeoutConfig {
                read_timeout_ms: 2500,
                write_timeout_ms: 2500,
            }
        }, 
        Some(
            DiagServerAdvancedOptions { // Advanced server options
                global_tp_id: 0,
                tester_present_interval_ms: 2000,
                tester_present_require_response: true,
                global_session_control: false,
                tp_ext_id: None,
                command_cooldown_ms: 100,
            }
        )
    ).unwrap();

    // Register hook for when ECU responsds with RequestCorrectlyReceivedResponsePending
    diag_server.register_waiting_hook(|| { ecu_waiting_hook() });
    diag_server.register_send_complete_hook(|| { tx_ok_hook() });
    // Set diag session mode
    let res = diag_server.kwp_set_session(KwpSessionType::ExtendedDiagnostics);
    println!("{:?}", res);
    loop {
        std::thread::sleep(Duration::from_millis(1000));
    }


}