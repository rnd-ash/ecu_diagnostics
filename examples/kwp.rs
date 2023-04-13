use std::time::Duration;

use ecu_diagnostics::{hardware::{HardwareScanner, self}, channel::{self, IsoTPSettings}, kwp2000::{Kwp2000Protocol, KwpSessionType}, dynamic_diag::{DiagServerBasicOptions, DiagServerAdvancedOptions, TimeoutConfig, DiagProtocol, DynamicDiagSession, DiagSessionMode}};

extern crate ecu_diagnostics;

fn ecu_waiting_hook() {
    println!("Called hook! ECU is processing our request");
}

fn tx_ok_hook() {
    println!("Called hook! Data was sent OK!");
}

fn main() {
    env_logger::init();
    let dev = ecu_diagnostics::hardware::socketcan::SocketCanScanner::new();
    let d = dev.open_device_by_name("can0").unwrap();
    
    let mut protocol = Kwp2000Protocol::new();
    println!("Diagnostic server is {}!", protocol.get_protocol_name());
    // Register a custom diagnostic session with the protocol (Usually OEM specific)
    protocol.register_session_type(DiagSessionMode {
        id: 0x93,
        tp_require: true,
        name: "SuperSecretDiagMode".into(),
    });

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

    // This call would work for KWP or UDS, not OBD2 as OBD2 has no form of 'session control'
    if let Some(mode) = diag_server.get_current_diag_mode() {
        println!("ECU is currently in '{}' diagnostic mode (0x{:02X?}). Tester present being sent?: {}", mode.name, mode.id, mode.tp_require);
    }

    // Register hook for when ECU responsds with RequestCorrectlyReceivedResponsePending
    diag_server.register_waiting_hook(|| { ecu_waiting_hook() });
    // Register hook for when our requests are sent to the ECU, but we have not got a response. Usually
    // this can be used to just let the program know Tx was OK!
    diag_server.register_send_complete_hook(|| { tx_ok_hook() });
    // Set diag session mode
    let res = diag_server.kwp_set_session(KwpSessionType::ExtendedDiagnostics);
    println!("Into extended diag mode result: {:?}", res);
    // Now check diag session mode, should be extended
    if let Some(mode) = diag_server.get_current_diag_mode() {
        println!("ECU is currently in '{}' diagnostic mode (0x{:02X?}). Tester present being sent?: {}", mode.name, mode.id, mode.tp_require);
    }

    // TODO - Emulate this call somehow for the sake of examples
    let res = diag_server.kwp_set_session(KwpSessionType::Custom { id: 0x93 }); // Same ID as what we registered at the start
    println!("Into special diag mode result: {:?}", res);
    if let Some(mode) = diag_server.get_current_diag_mode() {
        println!("ECU is currently in '{}' diagnostic mode (0x{:02X?}). Tester present being sent?: {}", mode.name, mode.id, mode.tp_require);
    }
    loop {
        // TP will be sent in this mode forever
        std::thread::sleep(Duration::from_millis(1000));
    }
}