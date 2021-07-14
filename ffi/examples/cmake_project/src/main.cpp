#include <iostream>
#include "ecu_diagnostics_ffi.hpp"

using namespace ecu_diagnostics;

std::string print_array_pretty(const uint8_t *arr, uint32_t size) {
    char* buff = new char[size*3 + 5];

    int len = 0;
    for(int i = 0; i < size; i++) {
        len += sprintf(buff+len, "%02X ", arr[i]);
    }
    std::string ret = buff;
    delete[] buff;
    return ret;
}

CallbackHandlerResult handle_isotp_config(IsoTPSettings cfg) {
    printf("\nSet ISO-TP config called! Configuration:\n");
    printf("Min separation time: %d\n", cfg.st_min);
    printf("Block size: %d\n", cfg.block_size);
    printf("CAN Speed: %d\n", cfg.can_speed);
    printf("CAN Ext Addressing? %s\n", cfg.can_use_ext_addr ? "Yes" : "No");
    printf("ISO-TP Ext Addressing? %s\n", cfg.extended_addressing ? "Yes" : "No");
    printf("Frame padding?: %s\n", cfg.pad_frame ? "Yes" : "No");
    /*
    opts.pad_frame = true;
    opts.st_min = 8;
     */
    return CallbackHandlerResult::OK;
}

CallbackHandlerResult handle_open(){
    printf("\nOpen called!\n");
    return CallbackHandlerResult::OK;
}
CallbackHandlerResult handle_close(){
    printf("\nClose called!\n");
    return CallbackHandlerResult::OK;
}

CallbackHandlerResult handle_clear_tx(){
    printf("\nClear Tx buffers called!\n");
    return CallbackHandlerResult::OK;
}

CallbackHandlerResult handle_clear_rx(){
    printf("\nClear Rx buffers called!\n");
    return CallbackHandlerResult::OK;
}

CallbackHandlerResult handle_write(CallbackPayload tx, uint32_t timeout){
    printf("\nWrite called! Data: { Dest-Addr: 0x%04X, data: [%s], timeout_ms: %d }\n", tx.addr, print_array_pretty(tx.data, tx.data_len).c_str(), timeout);
    return CallbackHandlerResult::OK;
}

CallbackHandlerResult handle_read(CallbackPayload *rx, uint32_t timeout){
    printf("\nRead called!\n");
    return CallbackHandlerResult::OK;
}

CallbackHandlerResult handle_set_ids(uint32_t send, uint32_t recv){
    printf("\nSet IDs called. Send: 0x%04x, Recv: 0x%04X\n", send, recv);
    return CallbackHandlerResult::OK;
}

int main() {
    // Base handler
    BaseChannelCallbackHandler base_handle = {};
    base_handle.open_callback = handle_open;
    base_handle.close_callback = handle_close;
    base_handle.clear_rx_callback = handle_clear_rx;
    base_handle.clear_tx_callback = handle_clear_tx;
    base_handle.read_bytes_callback = handle_read;
    base_handle.set_ids_callback = handle_set_ids;
    base_handle.write_bytes_callback = handle_write;

    // ISO-TP specific
    IsoTpChannelCallbackHandler iso_tp = {};
    iso_tp.base = base_handle;
    iso_tp.set_iso_tp_cfg_callback = handle_isotp_config;


    // Configure ISO-TP options
    IsoTPSettings opts = {};
    opts.block_size = 20;
    opts.can_speed = 500000;
    opts.can_use_ext_addr = false;
    opts.extended_addressing = false;
    opts.pad_frame = true;
    opts.st_min = 8;

    // Configure UDS server settings
    UdsServerOptions server_opts = {};
    server_opts.global_tp_id = 0x00;
    server_opts.read_timeout_ms = 1000;
    server_opts.write_timeout_ms = 1000;
    server_opts.recv_id = 0x07E8;
    server_opts.send_id = 0x07E0;
    server_opts.tester_present_interval_ms = 2500;
    server_opts.tester_present_require_response = true;

    // Register ISO-TP data handler
    register_isotp_callback(iso_tp);

    // Now start the UDS server!
    DiagServerResult server_status = create_uds_server_over_isotp(server_opts, opts);

    if (server_status == DiagServerResult::OK) {
        // Server is running! Lets execute some commands!
        printf("UDS Server open! Sending command\n");

        UdsPayload start_diag_req = UdsPayload{};
        uint8_t args[0x03]; // Extended session mode
        start_diag_req.sid = UDSCommand::DiagnosticSessionControl;
        start_diag_req.args_len = 0x01;
        start_diag_req.args_ptr = args;

        // Execute the command
        DiagServerResult cmd_result = send_payload_uds(&start_diag_req, true);
        if (cmd_result == DiagServerResult::OK) {
            printf("ECU is now in extended diagnostic session mode!\n");
        } else if (cmd_result == DiagServerResult::ECUError) {
            uint8_t err = get_ecu_error_code();
            printf("ECU Rejected request. Error code 0x%02X\n", err);
        } else {
            printf("Diag server error running request. Error code %d\n", cmd_result);
        }

    } else {
        printf("Error starting UDS server. Result: %d\n", server_status);
    }
}