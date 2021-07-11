#ifndef ECU_DIAG_H_
#define ECU_DIAG_H_

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

namespace ecu_diagnostics {

/// Callback handler result
enum class CallbackHandlerResult {
  /// Everything OK
  OK = 0,
  /// Read timeout
  ReadTimeout = 2,
  /// Write timeout
  WriteTimeout = 3,
  /// Internal API error
  APIError = 4,
  /// Callback already exists. Cannot register a new one
  CallbackAlreadyExists = 5,
};

/// FFI Diagnostic server response codes
enum class DiagServerResult {
  /// Operation OK
  OK = 0,
  /// Operation not supported by diagnostic server
  NotSupported = 1,
  /// ECU Responded with no data
  EmptyResponse = 2,
  /// ECU Responded with incorrect SID
  WrongMessage = 3,
  /// Internal diagnostic server is not running. Must have encountered a critical error
  ServerNotRunning = 4,
  /// ECU Response was of invalid length
  InvalidResponseLength = 5,
  /// No Callback handler registered
  NoHandler = 6,
  /// Diagnostic server is already running, cannot create a new one
  ServerAlreadyRunning = 7,
  /// No diagnostic server to register the request. Call
  NoDiagnosticServer = 8,
  /// ECU responded with an error, call [get_ecu_error_code]
  /// to retrieve the NRC from the ECU
  ECUError = 98,
  /// Callback handler error
  HandlerError = 99,
};

/// UDS Command Service IDs
enum class UDSCommand : uint8_t {
  /// Diagnostic session control. See [diagnostic_session_control]
  DiagnosticSessionControl = 16,
  /// ECU Reset. See [ecu_reset]
  ECUReset = 17,
  /// Security access. See [security_access]
  SecurityAccess = 39,
  /// Controls communication functionality of the ECU
  CommunicationControl = 40,
  /// Tester present command.
  TesterPresent = 62,
  AccessTimingParameters = 131,
  SecuredDataTransmission = 132,
  ControlDTCSettings = 133,
  ResponseOnEvent = 134,
  LinkControl = 135,
  ReadDataByIdentifier = 34,
  ReadMemoryByAddress = 35,
  ReadScalingDataByIdentifier = 36,
  ReadDataByPeriodicIdentifier = 42,
  DynamicallyDefineDataIdentifier = 44,
  WriteDataByIdentifier = 46,
  WriteMemoryByAddress = 61,
  ClearDiagnosticInformation = 20,
  ReadDTCInformation = 25,
  InputOutputControlByIdentifier = 47,
  RoutineControl = 49,
  RequestDownload = 52,
  RequestUpload = 53,
  TransferData = 54,
  RequestTransferExit = 55,
};

/// Callback handler payload
struct CallbackPayload {
  /// Target send address
  uint32_t addr;
  /// Data size
  uint32_t data_len;
  /// Data
  uint8_t data[4096];
};

/// Callback handler for base channel to allow access via FFI
struct BaseChannelCallbackHandler {
  /// Callback when [BaseChannel::open] is called
  CallbackHandlerResult (*open_callback)();
  /// Callback when [BaseChannel::close] is called
  CallbackHandlerResult (*close_callback)();
  /// Callback when [BaseChannel::write_bytes] is called
  CallbackHandlerResult (*write_bytes_callback)(CallbackPayload write_payload, uint32_t write_timeout_ms);
  /// Callback when [BaseChannel::read_bytes] is called
  CallbackHandlerResult (*read_bytes_callback)(CallbackPayload *read_payload, uint32_t read_timeout_ms);
  /// Callback when [BaseChannel::clear_tx_buffer] is called
  CallbackHandlerResult (*clear_tx_callback)();
  /// Callback when [BaseChannel::clear_rx_buffer] is called
  CallbackHandlerResult (*clear_rx_callback)();
  /// Callback when [BaseChannel::set_ids] is called
  CallbackHandlerResult (*set_ids_callback)(uint32_t send, uint32_t recv);
};

/// ISO-TP configuration options
struct IsoTPSettings {
  /// Block size
  uint8_t block_size;
  /// Minimum separation time between CAN Frames (In milliseconds)
  uint8_t st_min;
  /// Use extended ISO-TP addressing
  bool extended_addressing;
  /// Pad frames over ISO-TP if data size < 8
  bool pad_frame;
  /// Baud rate of the CAN Network
  uint32_t can_speed;
  /// Does the CAN Network support extended addressing (29bit) or standard addressing (11bit)
  bool can_use_ext_addr;
};

/// Callback handler for [IsoTPChannel]
struct IsoTpChannelCallbackHandler {
  /// Base handler
  BaseChannelCallbackHandler base;
  /// Callback when [IsoTPChannel::set_iso_tp_cfg] is called
  CallbackHandlerResult (*set_iso_tp_cfg_callback)(IsoTPSettings cfg);
};

/// UDS server options
struct UdsServerOptions {
  /// ECU Send ID
  uint32_t send_id;
  /// ECU Receive ID
  uint32_t recv_id;
  /// Read timeout in ms
  uint32_t read_timeout_ms;
  /// Write timeout in ms
  uint32_t write_timeout_ms;
  /// Optional global address to send tester-present messages to
  /// Set to 0 if not in use
  uint32_t global_tp_id;
  /// Tester present minimum send interval in ms
  uint32_t tester_present_interval_ms;
  /// Configures if the diagnostic server will poll for a response from tester present.
  bool tester_present_require_response;
};

/// Payload to send to the UDS server
struct UdsPayload {
  /// Service ID
  UDSCommand sid;
  /// Argument length
  uint32_t args_len;
  /// Arguments
  uint8_t args[4095];
};

extern "C" {

/// Register an ISO-TP callback
void register_isotp_callback(IsoTpChannelCallbackHandler cb);

/// Delete an ISO-TP callback
void destroy_isotp_callback();

/// Gets the last ECU negative response code
uint8_t get_ecu_error_code();

/// Creates a new UDS diagnostic server using an ISO-TP callback handler
DiagServerResult create_uds_server_over_isotp(UdsServerOptions settings, IsoTPSettings iso_tp_opts);

/// Sends a payload to the UDS server, attempts to get the ECUs response
///
/// ## Parameters
/// * payload - Payload to send to the ECU. If the ECU responds OK, then this payload
/// will be replaced by the ECUs response
///
/// * response_require - If set to false, no response will be read from the ECU.
///
/// ## Notes
///
/// Due to restrictions, the payload SID in the response message will match the original SID,
/// rather than SID + 0x40.
DiagServerResult send_payload_uds(UdsPayload *payload, bool response_require);

/// Destroys an existing UDS server
void destroy_uds_server();

} // extern "C"

} // namespace ecu_diagnostics

#endif // ECU_DIAG_H_
