pub mod kwp2000;
pub mod obd2;
pub mod uds;

extern crate alloc;
use alloc::{vec::Vec};


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagError {
    /// The Diagnostic server does not support the request
    NotSupported,
    /// IO Error when reading or writing to the ECU
    IOError,
    /// Timeout occurred
    Timeout,
    /// Diagnostic error code from the ECU itself
    ECUError(u8),
    /// Response empty
    EmptyResponse,
    /// ECU Responded but send a message that wasn't a reply for the sent message
    WrongMessage,
    /// Diagnostic server terminated!?
    ServerNotRunning
}

pub type DiagServerResult<T> = Result<T, DiagError>;


pub trait BaseChannel: Send + Sync {

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

    fn read_write_bytes(&mut self, buffer: &[u8], write_timeout_ms: u32, read_timeout_ms: u32) -> DiagServerResult<Vec<u8>> {
        self.write_bytes(buffer, write_timeout_ms)?;
        self.read_bytes(read_timeout_ms)
    }

    fn clear_rx_buffer(&mut self) -> DiagServerResult<()>;
    fn clear_tx_buffer(&mut self) -> DiagServerResult<()>;
}

impl Clone for Box<dyn BaseChannel> {
    fn clone(&self) -> Self {
        self.clone_base()
    }
}

/// Utilize the ISO15765-2 protocol over CANBUS
pub trait IsoTPChannel: BaseChannel {
    /// Configures the ISO-TP Channel
    /// 
    /// ## Parameters
    /// * block_size - The ISO-TP block size 
    /// * st_min - The ISO-TP minimum separation time (in milliseconds)
    fn configure_iso_tp(&mut self, cfg: IsoTPSettings) -> DiagServerResult<()>;

    fn clone_isotp(&self) -> Box<dyn IsoTPChannel>;

    fn into_base(&self) -> Box<dyn BaseChannel>;
}

impl Clone for Box<dyn IsoTPChannel> {
    fn clone(&self) -> Self {
        self.clone_isotp()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IsoTPSettings {
    block_size: u8,
    st_min: u8
}

impl Default for IsoTPSettings {
    fn default() -> Self {
        Self {
            block_size: 8,
            st_min: 20
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub struct DTC {
    pub id: u32,
    pub state: DTCState,
    pub mil_on: bool
}

#[derive(Debug, Copy, Clone)]
pub enum DTCState {

}