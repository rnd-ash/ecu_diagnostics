pub mod kwp2000;
pub mod obd2;
pub mod uds;

extern crate alloc;
use alloc::vec::Vec;

mod helpers;

pub type DiagServerResult<T> = Result<T, DiagError>;

#[derive(Debug)]
/// Diagnostic server error
pub enum DiagError {
    /// The Diagnostic server does not support the request
    NotSupported,
    /// IO Error when reading or writing to the ECU
    IOError(std::io::Error),
    /// Timeout occurred
    Timeout,
    /// Diagnostic error code from the ECU itself
    ECUError(u8),
    /// Response empty
    EmptyResponse,
    /// ECU Responded but send a message that wasn't a reply for the sent message
    WrongMessage,
    /// Diagnostic server terminated!?
    ServerNotRunning,
    /// ECU Responded with a message, but the length was incorrect
    InvalidResponseLength,
}

/// Base trait for interfacing with an ECU.
/// This trait allows you to write or read bytes from an ECUs interface
pub trait BaseChannel: Send + Sync {
    /// Clones this trait's box
    fn clone_base(&self) -> Box<dyn BaseChannel>;

    /// Sets the baud rate of the channel
    ///
    /// ## Parameters
    /// * baud - The baud rate of the channel
    fn set_baud(&mut self, baud: u32) -> DiagServerResult<()>;
    /// Configures the diagnostic channel with specific IDs for configuring the diagnostic server
    ///
    /// ## Parameters
    /// * send - Send ID (ECU will listen for data with this ID)
    /// * recv - Receiving ID (ECU will send data with this ID)
    /// * global_tp_id - Optional ID for global tester present messages. Required for certain ECUs
    fn set_ids(&mut self, send: u32, recv: u32, global_tp_id: Option<u32>) -> DiagServerResult<()>;

    /// Attempts to read bytes from the channel.
    ///
    /// ## Parameters
    /// * timeout_ms - Timeout for reading bytes. If a value of 0 is used, it instructs the channel to immediately
    /// return with whatever was in its receiving buffer
    fn read_bytes(&mut self, timeout_ms: u32) -> DiagServerResult<Vec<u8>>;

    /// Attempts to write bytes to the channel
    ///
    /// ## Parameters
    /// * buffer - The buffer of bytes to write to the channel
    /// * timeout_ms - Timeout for writing bytes. If a value of 0 is used, it tells the channel to write without checking if
    /// data was actually written.
    fn write_bytes(&mut self, buffer: &[u8], timeout_ms: u32) -> DiagServerResult<()>;

    /// Attempts to write bytes to the channel, then listen for the channels response
    ///
    /// ## Parameters
    /// * buffer - The buffer of bytes to write to the channel as the request
    /// * write_timeout_ms - Timeout for writing bytes. If a value of 0 is used, it tells the channel to write without checking if
    /// data was actually written.
    /// * read_timeout_ms - Timeout for reading bytes. If a value of 0 is used, it instructs the channel to immediately
    /// return with whatever was in its receiving buffer
    fn read_write_bytes(
        &mut self,
        buffer: &[u8],
        write_timeout_ms: u32,
        read_timeout_ms: u32,
    ) -> DiagServerResult<Vec<u8>> {
        self.write_bytes(buffer, write_timeout_ms)?;
        self.read_bytes(read_timeout_ms)
    }

    /// Tells the channel to clear its Rx buffer
    fn clear_rx_buffer(&mut self) -> DiagServerResult<()>;

    /// Tells the channel to clear its Tx buffer
    fn clear_tx_buffer(&mut self) -> DiagServerResult<()>;
}

impl Clone for Box<dyn BaseChannel> {
    fn clone(&self) -> Self {
        self.clone_base()
    }
}

/// Basic diagnostic server settings
pub trait BaseServerSettings {
    /// Gets the write timeout for sending messages to the servers channel
    fn get_write_timeout_ms(&self) -> u32;
    /// Gets the read timeout for reading response messages from the servers channel
    fn get_read_timeout_ms(&self) -> u32;
}

/// Basic diagnostic server payload
pub trait BaseServerPayload {
    /// Gets the payload portion of the diagnostic message (Not including the SID)
    fn get_payload(&self) -> &[u8];
    /// Gets the SID (Service ID) byte from the payload
    fn get_sid_byte(&self) -> u8;
    /// Gets the entire message as a byte array. This is what is sent to the ECU
    fn to_bytes(&self) -> &[u8];
    /// Boolean indicating if the diagnostic server should poll the ECU for a response after sending the payload
    fn requires_response(&self) -> bool;
}

/// Handler for various events within the diagnostic server. This is useful for logging
pub trait DiagServerLogger<SessionType, RequestType>: Send + Sync {
    /// Called when the diagnostic server encounters a critical error, and cannot continue operation
    fn on_critical_error(&self, err_desc: &str);
    /// Called when the server exists
    fn on_server_exit(&self);
    /// Called when there is an error sending a tester present message to the ECU.
    /// In the event this occurs, the ECU could exit an extended diagnostic session
    fn on_tester_present_error(&self, err: DiagError);
    /// Called when the diagnostic session mode changes
    fn on_diag_session_change(&self, res: DiagServerResult<SessionType>);
    /// Called when the server starts
    fn on_server_start(&self);
    /// Called when the server receives a request payload from the client
    fn on_request(&self, req: RequestType);
    /// Called when the server sends a response payload back to the client
    fn on_respond(&self, res: DiagServerResult<Vec<u8>>);
}

/// Extended trait for [BaseChannel] when utilizing ISO-TP to send data to the ECU
pub trait IsoTPChannel: BaseChannel {
    /// Configures the ISO-TP Channel
    ///
    /// ## Parameters
    /// * block_size - The ISO-TP block size
    /// * st_min - The ISO-TP minimum separation time (in milliseconds)
    fn configure_iso_tp(&mut self, cfg: IsoTPSettings) -> DiagServerResult<()>;

    /// Clones this box
    fn clone_isotp(&self) -> Box<dyn IsoTPChannel>;

    /// Downcasts this to a [BaseChannel]
    fn into_base(&self) -> Box<dyn BaseChannel>;
}

impl Clone for Box<dyn IsoTPChannel> {
    fn clone(&self) -> Self {
        self.clone_isotp()
    }
}

#[derive(Debug, Copy, Clone)]
/// ISO-TP configuration options
pub struct IsoTPSettings {
    /// Block size
    pub block_size: u8,
    /// Minimum separation time between CAN Frames (In milliseconds)
    pub st_min: u8,
    /// Use extended ISO-TP addressing (NOT EXTENDED CAN)
    pub extended_addressing: bool,
    /// Pad frames over ISO-TP if data size < 8
    pub pad_frame: bool,
}

impl Default for IsoTPSettings {
    fn default() -> Self {
        Self {
            block_size: 8,
            st_min: 20,
            extended_addressing: false,
            pad_frame: true,
        }
    }
}

/*
#[derive(Debug, Copy, Clone)]
pub struct DTC {
    pub id: u32,
    pub state: DTCState,
    pub mil_on: bool
}

#[derive(Debug, Copy, Clone)]
pub enum DTCState {

}
*/
