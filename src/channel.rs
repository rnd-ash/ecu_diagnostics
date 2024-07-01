//! Module for logical communication channels with an ECU
//!
//! Currently, the following channel types are defined:
//! * [PayloadChannel] - Basic channel, all channels inherit this trait
//! * [IsoTPChannel] - IsoTP (ISO15765) channel

use std::{
    borrow::BorrowMut,
    sync::{mpsc, Arc, Mutex, PoisonError},
};

#[cfg(all(feature = "socketcan", target_os = "linux"))]
use socketcan::{CanDataFrame, EmbeddedFrame, ExtendedId, Id, StandardId};

use crate::hardware::HardwareError;

/// Communication channel result
pub type ChannelResult<T> = Result<T, ChannelError>;

#[derive(Debug, Clone, thiserror::Error)]
/// Error produced by a communication channel
pub enum ChannelError {
    /// Underlying IO Error with channel
    #[error("Device IO error")]
    IOError(
        #[from]
        #[source]
        Arc<std::io::Error>,
    ),
    /// Timeout when writing data to the channel
    #[error("Write timeout")]
    WriteTimeout,
    /// Timeout when reading from the channel
    #[error("Read timeout")]
    ReadTimeout,
    /// The channel's Rx buffer is empty. Only applies when read timeout is 0
    #[error("No data in receive buffer")]
    BufferEmpty,
    /// The channels Tx buffer is full
    #[error("Send buffer is full")]
    BufferFull,
    /// Unsupported channel request
    #[error("Device unsupported request")]
    UnsupportedRequest,
    /// The interface is not open
    #[error("Interface was not opened before request")]
    InterfaceNotOpen,
    /// Underlying API error with hardware
    #[error("Device hardware API error")]
    HardwareError(
        #[from]
        #[source]
        HardwareError,
    ),
    /// Channel not configured prior to opening
    #[error("Channel configuration error")]
    ConfigurationError,
    /// Other Channel error
    #[error("Unknown channel error: {0}")]
    Other(String),
}

impl<T> From<PoisonError<T>> for ChannelError {
    fn from(err: PoisonError<T>) -> Self {
        ChannelError::HardwareError(HardwareError::from(err))
    }
}

impl<T> From<PoisonError<T>> for HardwareError {
    fn from(_x: PoisonError<T>) -> Self {
        HardwareError::APIError {
            code: 99,
            desc: "PoisonError".into(),
        }
    }
}

impl From<mpsc::RecvError> for HardwareError {
    fn from(e: mpsc::RecvError) -> Self {
        HardwareError::APIError {
            code: 98,
            desc: e.to_string(),
        }
    }
}

impl From<mpsc::RecvError> for ChannelError {
    fn from(err: mpsc::RecvError) -> Self {
        ChannelError::HardwareError(HardwareError::from(err))
    }
}

impl From<mpsc::RecvTimeoutError> for ChannelError {
    fn from(_err: mpsc::RecvTimeoutError) -> Self {
        ChannelError::WriteTimeout // Only used for writing
    }
}

impl<T> From<mpsc::SendError<T>> for HardwareError {
    fn from(e: mpsc::SendError<T>) -> Self {
        HardwareError::APIError {
            code: 98,
            desc: e.to_string(),
        }
    }
}

impl<T> From<mpsc::SendError<T>> for ChannelError {
    fn from(err: mpsc::SendError<T>) -> Self {
        ChannelError::HardwareError(HardwareError::from(err))
    }
}

/// A payload channel is a way for a device to have a bi-directional communication
/// link with a specific ECU
pub trait PayloadChannel: Send + Sync {
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
    /// * addr - Target address of the message
    /// * ext_id - Optional extended address of the message
    /// * buffer - The buffer of bytes to write to the channel
    /// * timeout_ms - Timeout for writing bytes. If a value of 0 is used, it tells the channel to write without checking if
    /// data was actually written.
    fn write_bytes(
        &mut self,
        addr: u32,
        ext_id: Option<u8>,
        buffer: &[u8],
        timeout_ms: u32,
    ) -> ChannelResult<()>;

    /// Attempts to write bytes to the channel, then listen for the channels response
    ///
    /// ## Parameters
    /// * Target address of the message
    /// * ext_id - Optional extended address of the message
    /// * buffer - The buffer of bytes to write to the channel as the request
    /// * write_timeout_ms - Timeout for writing bytes. If a value of 0 is used, it tells the channel to write without checking if
    /// data was actually written.
    /// * read_timeout_ms - Timeout for reading bytes. If a value of 0 is used, it instructs the channel to immediately
    /// return with whatever was in its receiving buffer
    fn read_write_bytes(
        &mut self,
        addr: u32,
        ext_id: Option<u8>,
        buffer: &[u8],
        write_timeout_ms: u32,
        read_timeout_ms: u32,
    ) -> ChannelResult<Vec<u8>> {
        self.write_bytes(addr, ext_id, buffer, write_timeout_ms)?;
        self.read_bytes(read_timeout_ms)
    }

    /// Tells the channel to clear its Rx buffer.
    /// This means all pending messages to be read should be wiped from the devices queue,
    /// such that [PayloadChannel::read_bytes] does not read them
    fn clear_rx_buffer(&mut self) -> ChannelResult<()>;

    /// Tells the channel to clear its Tx buffer.
    /// This means all messages that are queued to be sent to the ECU should be wiped.
    fn clear_tx_buffer(&mut self) -> ChannelResult<()>;
}

/// Extended trait for [PayloadChannel] when utilizing ISO-TP to send data to the ECU
pub trait IsoTPChannel: PayloadChannel {
    /// Sets the ISO-TP specific configuration for the Channel
    ///
    /// ## Parameters
    /// * The configuration of the ISO-TP Channel
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()>;
}

/// A PacketChannel is a way for a device to send and receive individual network packets
/// across an ECU network. Unlike [PayloadChannel], this channel type
/// is unfiltered, so all network traffic may be visible, and filtering should be done
/// in software. Most of the protocols that implement [PayloadChannel] are actually higher-level
/// PacketChannels which use multiple packets to send larger payloads. Such is the case with ISO-TP over CAN.
pub trait PacketChannel<T: Packet>: Send + Sync {
    /// Opens the channel, from this point forward,
    /// the network filter will be applied to be fully open
    /// so data has to be polled rapidly to avoid a driver's
    /// internal buffer from filling up rapidly
    fn open(&mut self) -> ChannelResult<()>;

    /// Closes the channel. Once closed, no more traffic
    /// can be polled or written to the channel.
    fn close(&mut self) -> ChannelResult<()>;

    /// Writes a list of packets to the raw interface
    fn write_packets(&mut self, packets: Vec<T>, timeout_ms: u32) -> ChannelResult<()>;
    /// Reads a list of packets from the raw interface
    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> ChannelResult<Vec<T>>;

    /// Tells the channel to clear its Rx buffer.
    /// This means all pending messages to be read should be wiped from the devices queue,
    /// such that [PayloadChannel::read_bytes] does not read them
    fn clear_rx_buffer(&mut self) -> ChannelResult<()>;

    /// Tells the channel to clear its Tx buffer.
    /// This means all messages that are queued to be sent to the ECU should be wiped.
    fn clear_tx_buffer(&mut self) -> ChannelResult<()>;
}

/// Packet channel for sending and receiving individual CAN Frames
pub trait CanChannel: PacketChannel<CanFrame> {
    /// Sets the CAN network configuration
    fn set_can_cfg(&mut self, baud: u32, use_extended: bool) -> ChannelResult<()>;
}

impl<T: PayloadChannel + ?Sized> PayloadChannel for Box<T> {
    fn open(&mut self) -> ChannelResult<()> {
        T::open(self)
    }

    fn close(&mut self) -> ChannelResult<()> {
        T::close(self)
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        T::set_ids(self, send, recv)
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        T::read_bytes(self, timeout_ms)
    }

    fn write_bytes(
        &mut self,
        addr: u32,
        ext_id: Option<u8>,
        buffer: &[u8],
        timeout_ms: u32,
    ) -> ChannelResult<()> {
        T::write_bytes(self, addr, ext_id, buffer, timeout_ms)
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        T::clear_rx_buffer(self)
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        T::clear_tx_buffer(self)
    }
}

impl<T: IsoTPChannel + ?Sized> IsoTPChannel for Box<T> {
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()> {
        T::set_iso_tp_cfg(self, cfg)
    }
}

impl<X: Packet, T: PacketChannel<X> + ?Sized> PacketChannel<X> for Box<T> {
    fn open(&mut self) -> ChannelResult<()> {
        T::open(self)
    }

    fn close(&mut self) -> ChannelResult<()> {
        T::close(self)
    }

    fn write_packets(&mut self, packets: Vec<X>, timeout_ms: u32) -> ChannelResult<()> {
        T::write_packets(self, packets, timeout_ms)
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> ChannelResult<Vec<X>> {
        T::read_packets(self, max, timeout_ms)
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        T::clear_rx_buffer(self)
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        T::clear_tx_buffer(self)
    }
}

impl<T: CanChannel + ?Sized> CanChannel for Box<T> {
    fn set_can_cfg(&mut self, baud: u32, use_extended: bool) -> ChannelResult<()> {
        T::set_can_cfg(self, baud, use_extended)
    }
}

impl<T: PayloadChannel + ?Sized> PayloadChannel for Arc<Mutex<T>> {
    fn open(&mut self) -> ChannelResult<()> {
        T::open(self.lock()?.borrow_mut())
    }

    fn close(&mut self) -> ChannelResult<()> {
        T::close(self.lock()?.borrow_mut())
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        T::set_ids(self.lock()?.borrow_mut(), send, recv)
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        T::read_bytes(self.lock()?.borrow_mut(), timeout_ms)
    }

    fn write_bytes(
        &mut self,
        addr: u32,
        ext_id: Option<u8>,
        buffer: &[u8],
        timeout_ms: u32,
    ) -> ChannelResult<()> {
        T::write_bytes(self.lock()?.borrow_mut(), addr, ext_id, buffer, timeout_ms)
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        T::clear_rx_buffer(self.lock()?.borrow_mut())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        T::clear_tx_buffer(self.lock()?.borrow_mut())
    }
}

impl<T: IsoTPChannel + ?Sized> IsoTPChannel for Arc<Mutex<T>> {
    fn set_iso_tp_cfg(&mut self, cfg: IsoTPSettings) -> ChannelResult<()> {
        T::set_iso_tp_cfg(self.lock()?.borrow_mut(), cfg)
    }
}

impl<X: Packet, T: PacketChannel<X> + ?Sized> PacketChannel<X> for Arc<Mutex<T>> {
    fn open(&mut self) -> ChannelResult<()> {
        T::open(self.lock()?.borrow_mut())
    }

    fn close(&mut self) -> ChannelResult<()> {
        T::close(self.lock()?.borrow_mut())
    }

    fn write_packets(&mut self, packets: Vec<X>, timeout_ms: u32) -> ChannelResult<()> {
        T::write_packets(self.lock()?.borrow_mut(), packets, timeout_ms)
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> ChannelResult<Vec<X>> {
        T::read_packets(self.lock()?.borrow_mut(), max, timeout_ms)
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        T::clear_rx_buffer(self.lock()?.borrow_mut())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        T::clear_tx_buffer(self.lock()?.borrow_mut())
    }
}

impl<T: CanChannel + ?Sized> CanChannel for Arc<Mutex<T>> {
    fn set_can_cfg(&mut self, baud: u32, use_extended: bool) -> ChannelResult<()> {
        T::set_can_cfg(self.lock()?.borrow_mut(), baud, use_extended)
    }
}

/// This trait is for packets that are used by [PacketChannel]
pub trait Packet: Send + Sync + Sized {
    /// Returns the address of the packet
    fn get_address(&self) -> u32;
    /// Returns the data of the packet
    fn get_data(&self) -> &[u8];
    /// Sets the address of the packet
    fn set_address(&mut self, address: u32);
    /// Sets the data of the packet
    fn set_data(&mut self, data: &[u8]);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// CAN Frame
pub struct CanFrame {
    id: u32,
    dlc: u8,
    data: [u8; 8],
    ext: bool,
}

unsafe impl Sync for CanFrame {}
unsafe impl Send for CanFrame {}

impl CanFrame {
    /// Creates a new CAN Frame given data and an ID.
    /// ## Parameters
    /// * id - The CAN ID of the packet
    /// * data - The data of the CAN packet
    /// * is_ext - Indication if the CAN packet shall use extended addressing
    ///
    /// NOTE: If `id` is greater than 0x7FF, extended addressing (29bit) will be enabled
    /// regardless of `is_ext`.
    ///
    /// Also, `data` will be limited to 8 bytes.
    pub fn new(id: u32, data: &[u8], is_ext: bool) -> Self {
        let max = std::cmp::min(8, data.len());
        let mut tmp = [0u8; 8];
        tmp[0..max].copy_from_slice(&data[0..max]);
        Self {
            id,
            dlc: max as u8,
            data: tmp,
            ext: is_ext,
        }
    }

    /// Returns true if the CAN Frame uses Extended (29bit) addressing
    pub fn is_extended(&self) -> bool {
        self.ext
    }
}

impl Packet for CanFrame {
    fn get_address(&self) -> u32 {
        self.id
    }

    fn get_data(&self) -> &[u8] {
        &self.data[0..self.dlc as usize]
    }

    fn set_address(&mut self, address: u32) {
        self.id = address
    }
    fn set_data(&mut self, data: &[u8]) {
        let max = std::cmp::min(8, data.len());
        self.data[0..max].copy_from_slice(&data[0..max]);
        self.dlc = max as u8;
    }
}

#[cfg(all(feature = "socketcan", target_os = "linux"))]
impl From<CanDataFrame> for CanFrame {
    fn from(value: CanDataFrame) -> Self {
        let (id, ext) = match value.id() {
            Id::Standard(id) => (id.as_raw() as u32, true),
            Id::Extended(id) => (id.as_raw(), false),
        };
        CanFrame::new(id, value.data(), ext)
    }
}

#[cfg(all(feature = "socketcan", target_os = "linux"))]
impl Into<CanDataFrame> for CanFrame {
    fn into(self) -> CanDataFrame {
        let id = match self.ext {
            true => Id::Extended(ExtendedId::new(self.get_address()).unwrap()),
            false => Id::Standard(StandardId::new(self.get_address() as u16).unwrap()),
        };
        CanDataFrame::new(id, self.get_data()).unwrap()
    }
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
    /// Extended addressing bytes
    /// order is Tx ext address, Rx ext address
    pub extended_addresses: Option<(u8, u8)>,
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
            extended_addresses: None,
            pad_frame: true,
            can_speed: 500_000,
            can_use_ext_addr: false,
        }
    }
}
