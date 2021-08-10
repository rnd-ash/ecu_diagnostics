use std::process::exit;

use socketcan_iface::SocketCANInterface;

mod sim_ecu;
mod socketcan_iface;

use ecu_diagnostics::{DiagError, DiagServerResult, channel::IsoTPSettings, kwp2000::{self, Kwp2000DiagnosticServer, Kwp2000ServerOptions, Kwp2000VoidHandler, read_ecu_identification::*, start_diagnostic_session, read_data_by_local_id::*}};

fn main() {
    // Vcan device
    let vcan0 = SocketCANInterface::new("can0".to_string());

    // Setup KWP server
    let kwp_server_settings = Kwp2000ServerOptions {
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

    let mut kwp_server: Kwp2000DiagnosticServer = match Kwp2000DiagnosticServer::new_over_iso_tp(
        kwp_server_settings,
        vcan0,
        isotp_cfg,
        Kwp2000VoidHandler,
    ) {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Error setting up UDS server!: {}. Aborting", e);
            exit(1)
        }
    };

    match start_diagnostic_session::set_diagnostic_session_mode(&mut kwp_server, start_diagnostic_session::SessionType::ExtendedDiagnostics) {
        Ok(_) => println!("ECU is now in Extended Diag mode!"),
        Err(e) => {
            if let DiagError::ECUError(x) = e {
                // Error from ECU. Query the error
                eprintln!(
                    "ECU Rejected the request: {:?}",
                    kwp2000::get_description_of_ecu_error(x)
                );
            } else {
                eprintln!("Error setting Extended Diag mode: {:?}. Aborting", e);
            }
        }
    }

    test_kwp_operation("read_dcs_identification", read_dcs_identification(&mut kwp_server));
    test_kwp_operation("read_dcx_mmc_identification", read_dcx_mmc_identification(&mut kwp_server));
    test_kwp_operation("read_original_vin", read_original_vin(&mut kwp_server));
    test_kwp_operation("read_diagnostic_variant_code", read_diagnostic_variant_code(&mut kwp_server));
    test_kwp_operation("read_current_vin", read_current_vin(&mut kwp_server));
    test_kwp_operation("read_calibration_id", read_calibration_id(&mut kwp_server));
    test_kwp_operation("read_cvn", read_cvn(&mut kwp_server));
    test_kwp_operation("read_ecu_code_fingerprint", read_ecu_code_fingerprint(&mut kwp_server));
    test_kwp_operation("read_ecu_data_fingerprint", read_ecu_data_fingerprint(&mut kwp_server));
    test_kwp_operation("read_ecu_code_software_id", read_ecu_code_software_id(&mut kwp_server));
    test_kwp_operation("read_ecu_data_software_id", read_ecu_data_software_id(&mut kwp_server));
    test_kwp_operation("read_ecu_boot_software_id", read_ecu_boot_software_id(&mut kwp_server));
    test_kwp_operation("read_ecu_boot_fingerprint", read_ecu_boot_fingerprint(&mut kwp_server));
    test_kwp_operation("read_ecu_development_data", read_ecu_development_data(&mut kwp_server));
    test_kwp_operation("read_ecu_serial_number", read_ecu_serial_number(&mut kwp_server));
    test_kwp_operation("read_ecu_dbcom_data", read_ecu_dbcom_data(&mut kwp_server));
    test_kwp_operation("read_ecu_os_version", read_ecu_os_version(&mut kwp_server));
    test_kwp_operation("read_ecu_reprogramming_fault_report", read_ecu_reprogramming_fault_report(&mut kwp_server));
    test_kwp_operation("read_ecu_vehicle_info", read_ecu_vehicle_info(&mut kwp_server));
    test_kwp_operation("read_ecu_flash_info_1", read_ecu_flash_info_1(&mut kwp_server));
    test_kwp_operation("read_ecu_flash_info_2", read_ecu_flash_info_2(&mut kwp_server));
    test_kwp_operation("read_system_diag_general_param_data", read_system_diag_general_param_data(&mut kwp_server));
    test_kwp_operation("read_system_diag_global_param_data", read_system_diag_global_param_data(&mut kwp_server));
    test_kwp_operation("read_ecu_configuration", read_ecu_configuration(&mut kwp_server));
    test_kwp_operation("read_diag_protocol_info", read_diag_protocol_info(&mut kwp_server));

    

    // If execution reaches here, Then ECU will be in extended diagnostic mode, allowing
    // for all UDS functionality. NOTE: This might trigger some warning lights on a cars
    // instrument cluster, as in diagnostic mode, certain ECUs stop behaving normally.
    // The car will return to normal once this program exists and the ECU goes back to normal.
}

#[inline(always)]
pub fn test_kwp_operation<T: std::fmt::Debug>(func: &str, res: DiagServerResult<T>) {
    match res {
        Ok(x) => println!("{} succeeded. Result: {:#?}", func, x),
        Err(err) => {
            if let DiagError::ECUError(e) = err {
                println!(
                    "ECU Rejected the request for '{}'! Error: {:?}",
                    func,
                    kwp2000::get_description_of_ecu_error(e)
                )
            } else {
                println!("Error executing '{}': {}", func, err);
            }
        }
    }
}

/*
Here we have a custom Event handler, which can log internal UDS Server events!
*/

/*

struct MyUdsEventHandler {}

impl ServerEventHandler<UDSSessionType, UdsCmd> for MyUdsEventHandler {
    fn on_event(&mut self, e: ecu_diagnostics::ServerEvent<UDSSessionType, UdsCmd>) {
        match e {
            ecu_diagnostics::ServerEvent::CriticalError { desc } => {
                eprintln!("Server encountered a critical error: {}", desc)
            }
            ecu_diagnostics::ServerEvent::ServerStart => println!("UDS Server start!"),
            ecu_diagnostics::ServerEvent::ServerExit => println!("UDS Server stop!"),
            ecu_diagnostics::ServerEvent::DiagModeChange { old, new } => {
                println!("ECU Session state changed from {:?} to {:?}", old, new);
            }
            ecu_diagnostics::ServerEvent::IncomingEvent(e) => {
                println!("Transmitting {:02X?}", e)
            }
            ecu_diagnostics::ServerEvent::OutgoingEvent(e) => {
                println!("Receiving {:02X?}", e)
            }
            ecu_diagnostics::ServerEvent::TesterPresentError(e) => {
                eprintln!("Tester-Present could not be sent to the ECU: {:?}", e)
            }
            ecu_diagnostics::ServerEvent::InterfaceCloseOnExitError(e) => {
                eprintln!(
                    "Diagnostic server couldn't terminate channel connection: {:?}",
                    e
                )
            }
        }
    }
}

*/
