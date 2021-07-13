use std::process::exit;

use socketcan_iface::SocketCANInterface;

mod sim_ecu;
mod socketcan_iface;

use ecu_diagnostics::{DiagError, DiagServerResult, ServerEventHandler, channel::{IsoTPSettings}, uds::{self, UdsCmd, UdsDiagnosticServer, UdsServerOptions, diagnostic_session_control::UDSSessionType}};

fn main() {
    // Vcan device
    let vcan0 = SocketCANInterface::new("can0".to_string());

    // Setup UDS server
    let uds_server_settings = UdsServerOptions {
        send_id: 0x07E0,
        recv_id: 0x07E8,
        read_timeout_ms: 1000,
        write_timeout_ms: 1000,
        global_tp_id: 0,
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
        match UdsDiagnosticServer::new_over_iso_tp(uds_server_settings, vcan0, isotp_cfg, MyUdsEventHandler{}) {
            Ok(server) => server,
            Err(e) => {
                eprintln!("Error setting up UDS server!: {}. Aborting", e);
                exit(1)
            }
        };

    match uds::diagnostic_session_control::set_extended_mode(&mut uds_server) {
        Ok(_) => println!("ECU is now in Extended Diag mode!"),
        Err(e) => {
            if let DiagError::ECUError(x) = e { // Error from ECU. Query the error
                eprintln!("ECU Rejected the request: {:?}", uds::get_description_of_ecu_error(x));
            } else {
                eprintln!("Error setting Extended Diag mode: {:?}. Aborting", e);
            }
        }
    }

    // If execution reaches here, Then ECU will be in extended diagnostic mode, allowing
    // for all UDS functionality. NOTE: This might trigger some warning lights on a cars
    // instrument cluster, as in diagnostic mode, certain ECUs stop behaving normally.
    // The car will return to normal once this program exists and the ECU goes back to normal.

    test_uds_operation("get_number_of_dtcs_by_status_mask", uds::read_dtc_information::get_number_of_dtcs_by_status_mask(&mut uds_server, 0xFF));
    test_uds_operation("get_dtcs_by_status_mask", uds::read_dtc_information::get_dtcs_by_status_mask(&mut uds_server, 0xFF));
    test_uds_operation("get_mirror_memory_dtcs_by_status_mask", uds::read_dtc_information::get_mirror_memory_dtcs_by_status_mask(&mut uds_server, 0xFF));
    test_uds_operation("get_number_of_mirror_memory_dtcs_by_status_mask", uds::read_dtc_information::get_number_of_mirror_memory_dtcs_by_status_mask(&mut uds_server, 0xFF));
    test_uds_operation("get_number_of_emissions_related_obd_dtcs_by_status_mask", uds::read_dtc_information::get_number_of_emissions_related_obd_dtcs_by_status_mask(&mut uds_server, 0xFF));
    test_uds_operation("get_emissions_related_obd_dtcs_by_status_mask", uds::read_dtc_information::get_emissions_related_obd_dtcs_by_status_mask(&mut uds_server, 0xFF));
    test_uds_operation("get_dtc_snapshot_record_by_dtc_number", uds::read_dtc_information::get_dtc_snapshot_record_by_dtc_number(&mut uds_server, 0xFFFF, 0xFF));
    test_uds_operation("get_dtc_snapshot_identification", uds::read_dtc_information::get_dtc_snapshot_identification(&mut uds_server));
    test_uds_operation("get_dtc_snapshot_record_by_record_number", uds::read_dtc_information::get_dtc_snapshot_record_by_record_number(&mut uds_server, 0x01));
    test_uds_operation("get_dtc_extended_data_record_by_dtc_number", uds::read_dtc_information::get_dtc_extended_data_record_by_dtc_number(&mut uds_server, 0xFFFF, 0xFF));
    test_uds_operation("get_mirror_memory_dtc_extended_data_record_by_dtc_number", uds::read_dtc_information::get_mirror_memory_dtc_extended_data_record_by_dtc_number(&mut uds_server, 0xFFFF, 0xFF));
    test_uds_operation("get_number_of_dtcs_by_severity_mask_record", uds::read_dtc_information::get_number_of_dtcs_by_severity_mask_record(&mut uds_server, 0xFF, 0xFF));
    test_uds_operation("get_dtcs_by_severity_mask_record", uds::read_dtc_information::get_dtcs_by_severity_mask_record(&mut uds_server, 0xFF, 0xFF));
    test_uds_operation("get_severity_information_of_dtc", uds::read_dtc_information::get_severity_information_of_dtc(&mut uds_server, 0xFFFFFF));
    test_uds_operation("get_supported_dtc", uds::read_dtc_information::get_supported_dtc(&mut uds_server));
    test_uds_operation("get_first_test_failed_dtc", uds::read_dtc_information::get_first_test_failed_dtc(&mut uds_server));
    test_uds_operation("get_first_confirmed_dtc", uds::read_dtc_information::get_first_confirmed_dtc(&mut uds_server));
    test_uds_operation("get_most_recent_test_failed_dtc", uds::read_dtc_information::get_most_recent_test_failed_dtc(&mut uds_server));
    test_uds_operation("get_most_recent_confirmed_dtc", uds::read_dtc_information::get_most_recent_confirmed_dtc(&mut uds_server));
    test_uds_operation("get_dtc_fault_detection_counter", uds::read_dtc_information::get_dtc_fault_detection_counter(&mut uds_server));
    test_uds_operation("get_dtc_with_permanent_status", uds::read_dtc_information::get_dtc_with_permanent_status(&mut uds_server));







    // UDS Server will be terminated and channel closed when UDSDiagnosticServer is dropped.
}

#[inline(always)]
pub fn test_uds_operation<T: std::fmt::Debug>(func: &str, res: DiagServerResult<T>) {
    match res {
        Ok(x) => println!("{} succeeded. Result: {:02X?}", func, x),
        Err(err) => {
            if let DiagError::ECUError(e) = err {
                println!("ECU Rejected the request for '{}'! Error: {:?}", func, uds::get_description_of_ecu_error(e))
            } else {
                println!("Error executing '{}': {}", func, err);
            }
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
}

/*
Here we have a custom Event handler, which can log internal UDS Server events!
*/

struct MyUdsEventHandler{}

impl ServerEventHandler<UDSSessionType, UdsCmd> for MyUdsEventHandler {
    fn on_event(&mut self, e: ecu_diagnostics::ServerEvent<UDSSessionType, UdsCmd>) {
        match e {
            ecu_diagnostics::ServerEvent::CriticalError { desc } => eprintln!("Server encountered a critical error: {}", desc),
            ecu_diagnostics::ServerEvent::ServerStart => println!("UDS Server start!"),
            ecu_diagnostics::ServerEvent::ServerExit => println!("UDS Server stop!"),
            ecu_diagnostics::ServerEvent::DiagModeChange { old, new } => {
                println!("ECU Session state changed from {:?} to {:?}", old, new);
            },
            ecu_diagnostics::ServerEvent::IncomingEvent(e) => {
                println!("Transmitting {:02X?}", e)
            },
            ecu_diagnostics::ServerEvent::OutgoingEvent(e) => {
                println!("Receiving {:02X?}", e)
            },
            ecu_diagnostics::ServerEvent::TesterPresentError(e) => {
                eprintln!("Tester-Present could not be sent to the ECU: {:?}", e)
            },
        }
    }
}