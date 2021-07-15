//! Module for logical communication channels with an ECU
//!
//! Currently, the following channel types are defined:
//! * [BaseChannel] - Basic channel, all channels inherit this trait
//! * [IsoTPChannel] - IsoTP (ISO15765) channel

/// Communication channel result
pub type ChannelResult<T> = Result<T, ChannelError>;

#[derive(Debug)]
/// Error produced by a communication channel
pub enum ChannelError {
    /// Underlying IO Error with channel
    IOError(std::io::Error),
    /// Timeout when writing data to the channel
    WriteTimeout,
    /// Timeout when reading from the channel
    ReadTimeout,
    /// The channel's Rx buffer is empty. Only applies when read timeout is 0
    BufferEmpty,
    /// The channels Tx buffer is full
    BufferFull,
    /// Unsupported channel request
    UnsupportedRequest,
    /// The interface is not open
    InterfaceNotOpen,
    /// Underlying API error with hardware
    APIError {
        /// Name of the API EG: 'socketCAN', 'Passthru'
        api_name: String,
        /// Internal API error code
        code: u8,
        /// API error description
        desc: String,
    },
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelError::IOError(e) => write!(f, "IO error: {}", e),
            ChannelError::UnsupportedRequest => write!(f, "unsupported channel request"),
            ChannelError::ReadTimeout => write!(f, "timeout reading from channel"),
            ChannelError::WriteTimeout => write!(f, "timeout writing to channel"),
            ChannelError::BufferFull => write!(f, "channel's Transmit buffer is full"),
            ChannelError::BufferEmpty => write!(f, "channel's Receive buffer is empty"),
            ChannelError::InterfaceNotOpen => write!(f, "channel's interface is not open"),
            ChannelError::APIError {
                api_name,
                code,
                desc,
            } => write!(f, "underlying {} API error ({}): {}", api_name, code, desc),
        }
    }
}

impl std::error::Error for ChannelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::IOError(io_err) = self {
            Some(io_err)
        } else {
            None
        }
    }
}

/// Base trait for interfacing with an ECU.
/// This trait allows you to write or read bytes from an ECUs interface
pub trait BaseChannel: Send + Sync {
    /// This function opens the interface.
    /// It is ONLY called after set_ids and any other configuration function
    fn open(&mut self) -> ChannelResult<()>;

    /// Closes and destroys the channel
    fn close(&mut self) -> ChannelResult<()>;

    /// Configures the diagnostic channel with specific IDs for configuring the diagnostic server
    ///
    /// ## Parameters
    /// * send - Send ID (ECU will listen for data with this ID)
    /// * recv - Receiving ID (ECU will send data with this ID)
    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()>;

    /// Attempts to read bytes from the channel.
    /// 
    /// The contents being read should not include any protocol related bytes,
    /// just the payload destined for the diagnostic application
    ///
    /// ## Parameters
    /// * timeout_ms - Timeout for reading bytes. If a value of 0 is used, it instructs the channel to immediately
    /// return with whatever was in its receiving buffer
    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>>;

    /// Attempts to write bytes to the channel.
    /// 
    /// The contents being sent will just be the raw payload being sent to the device,
    /// it is up to the implementor of this function to add related protocol bytes
    /// to the message where necessary.
    ///
    /// ## Parameters
    /// * Target address of the message
    /// * buffer - The buffer of bytes to write to the channel
    /// * timeout_ms - Timeout for writing bytes. If a value of 0 is used, it tells the channel to write without checking if
    /// data was actually written.
    fn write_bytes(&mut self, addr: u32, buffer: &[u8], timeout_ms: u32) -> ChannelResult<()>;

    /// Attempts to write bytes to the channel, then listen for the channels response
    ///
    /// ## Parameters
    /// * Target address of the message
    /// * buffer - The buffer of bytes to write to the channel as the request
    /// * write_timeout_ms - Timeout for writing bytes. If a value of 0 is used, it tells the channel to write without checking if
    /// data was actually written.
    /// * read_timeout_ms - Timeout for reading bytes. If a value of 0 is used, it instructs the channel to immediately
    /// return with whatever was in its receiving buffer
    fn read_write_bytes(
        &mut self,
        addr: u32,
        buffer: &[u8],
        write_timeout_ms: u32,
        read_timeout_ms: u32,
    ) -> ChannelResult<Vec<u8>> {
        self.write_bytes(addr, buffer, write_timeout_ms)?;
        self.read_bytes(read_timeout_ms)
    }

    /// Tells the channel to clear its Rx buffer.
    /// This means all pending messages to be read should be wiped from the devices queue,
    /// such that [BaseChannel::read_bytes] does not read them
    fn clear_rx_buffer(&mut self) -> ChannelResult<()>;

    /// Tells the channel to clear its Tx buffer.
    /// This means all messages that are queued to be sent to the ECU should be wiped.
    fn clear_tx_buffer(&mut self) -> ChannelResult<()>;
}

/// Extended trait for [BaseChannel] when utilizing ISO-TP to send data to the ECU
pub trait IsoTPChannel: BaseChannel {
    /// Sets the ISO-TP specific configuration for the Channel
    ///
    /// ## Parameters
    /// * The configuration of the ISO-TP Channel
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()>;
}

/// ISO-TP configuration options (ISO15765-2)
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct IsoTPSettings {
    /// ISO-TP Block size
    /// 
    /// This value indicates the number of CAN Frames to send in multi-frame messages,
    /// before sending or receiving a flow control message.
    /// 
    /// A value of 0 indicates send everything without flow control messages.
    /// 
    /// NOTE: This value might be overridden by the device's implementation of ISO-TP
    pub block_size: u8,
    /// Minimum separation time between Tx/Rx CAN Frames.
    /// 
    /// 3 ranges are accepted for this value:
    /// * 0x00 - Send without delay (ECU/Adapter will send frames as fast as the physical bus allows).
    /// * 0x01-0x7F - Send with delay of 1-127 milliseconds between can frames
    /// * 0xF1-0xF9 - Send with delay of 100-900 microseconds between can frames
    /// 
    /// NOTE: This value might be overridden by the device's implementation of ISO-TP
    pub st_min: u8,
    /// Use extended ISO-TP addressing
    pub extended_addressing: bool,
    /// Pad frames over ISO-TP if data size is less than 8.
    pub pad_frame: bool,
    /// Baud rate of the CAN Network
    pub can_speed: u32,
    /// Does the CAN Network support extended addressing (29bit) or standard addressing (11bit)
    pub can_use_ext_addr: bool,
}

impl Default for IsoTPSettings {
    fn default() -> Self {
        Self {
            block_size: 8,
            st_min: 20,
            extended_addressing: false,
            pad_frame: true,
            can_speed: 500_000,
            can_use_ext_addr: false,
        }
    }
}
