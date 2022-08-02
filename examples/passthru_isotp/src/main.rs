use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use ecu_diagnostics::uds::{UdsDiagnosticServer, UdsServerOptions, UdsVoidHandler};
use ecu_diagnostics::{self, uds};

use ecu_diagnostics::channel::{CanChannel, CanFrame, IsoTPSettings};
use ecu_diagnostics::hardware::passthru::*;
use ecu_diagnostics::hardware::{Hardware, HardwareScanner};

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

    let device = match passthru_scanner.open_device_by_name("Macchina M2 Under the dash") {
        Ok(d) => d,
        Err(e) => {
            println!("Error opening passthru device 0: {}", e);
            exit(1)
        }
    };

    let iso_tp_channel = match Hardware::create_iso_tp_channel(device) {
        Ok(d) => d,
        Err(e) => {
            println!("Error opening passthru device 0: {}", e);
            exit(1)
        }
    };

    let uds_settings = UdsServerOptions {
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
        extended_addresses: None,
        pad_frame: true,
        can_speed: 500000,
        can_use_ext_addr: false,
    };

    let mut uds_server = match UdsDiagnosticServer::new_over_iso_tp(
        uds_settings,
        iso_tp_channel,
        iso_tp_settings,
        UdsVoidHandler {},
    ) {
        Ok(s) => s,
        Err(e) => {
            println!("Error starting KWP2000 server: {}", e);
            exit(1)
        }
    };

    match uds::get_dtcs_by_status_mask(&mut uds_server, 0xFF) {
        Ok(dtcs) => {
            println!("DTCs: {:?}", dtcs)
        },
        Err(e) => println!("Error reading DTCs from ECU: {}", e)
    }
}
