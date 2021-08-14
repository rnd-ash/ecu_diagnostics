use std::process::exit;
use std::sync::{Arc, Mutex};

use ecu_diagnostics::{self, kwp2000};

use ecu_diagnostics::channel::IsoTPSettings;
use ecu_diagnostics::hardware::passthru::*;
use ecu_diagnostics::hardware::{Hardware, HardwareScanner};
use ecu_diagnostics::kwp2000::{Kwp2000DiagnosticServer, Kwp2000ServerOptions, Kwp2000VoidHandler};

/// A simple example of using a KWP2000 diagnostic server over ISO-TP
/// using a passthru adapter!

fn main() {
    let passthru_scanner = PassthruScanner::new();
    let list = passthru_scanner.list_devices();

    if list.len() == 0 {
        println!("No passthru devices found");
        exit(0);
    }

    println!("Found the following passthru devices:");
    for x in &list {
        println!(
            "{} by {} - Supports ISO-TP?: {}",
            x.name, x.vendor, x.capabilities.iso_tp
        );
    }

    let device = match passthru_scanner.open_device_by_index(0) {
        Ok(d) => d,
        Err(e) => {
            println!("Error opening passthru device 0: {}", e);
            exit(1)
        }
    };

    let iso_tp_channel = match PassthruDevice::create_iso_tp_channel(device) {
        Ok(c) => c,
        Err(e) => {
            println!("Error creating ISO-TP channel on passthru device 0: {}", e);
            exit(1)
        }
    };

    let kwp_settings = Kwp2000ServerOptions {
        send_id: 0x07E0,
        recv_id: 0x07E8,
        read_timeout_ms: 1000,
        write_timeout_ms: 1000,
        global_tp_id: 0,
        tester_present_interval_ms: 2000,
        tester_present_require_response: true,
    };

    let iso_tp_settings = IsoTPSettings {
        block_size: 8,
        st_min: 20,
        extended_addressing: false,
        pad_frame: true,
        can_speed: 500000,
        can_use_ext_addr: false,
    };

    let mut kwp_server = match Kwp2000DiagnosticServer::new_over_iso_tp(
        kwp_settings,
        iso_tp_channel,
        iso_tp_settings,
        Kwp2000VoidHandler {},
    ) {
        Ok(s) => s,
        Err(e) => {
            println!("Error starting KWP2000 server: {}", e);
            exit(1)
        }
    };

    // Put server into extended diag mode
    match kwp2000::start_diagnostic_session::set_diagnostic_session_mode(
        &mut kwp_server,
        kwp2000::start_diagnostic_session::SessionType::ExtendedDiagnostics,
    ) {
        Ok(_) => println!("ECU now in extended diagnostic mode!"),
        Err(e) => println!("Error executing extended diagnostic mode request: {}", e),
    }

    match kwp2000::read_dtc_by_status::read_stored_dtcs(
        &mut kwp_server,
        kwp2000::read_dtc_by_status::DTCRange::All,
    ) {
        Ok(res) => println!("List of DTCs stored on the ECU: {:#?}", res),
        Err(e) => println!("Error reading DTCs from ECU: {}", e),
    }
}
