//! Implements SLCAN device

use std::{
    collections::VecDeque,
    fmt::{Debug, Formatter, Result as FmtResult},
    io::{Read, Write},
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::Instant
};

use serial_rs::SerialPort;

use crate::{channel::{CanFrame, ChannelError, Packet}, hardware::{HardwareCapabilities, HardwareInfo}};

const MAX_PACKET_SIZE: usize = 32;
const HEX: [u8; 16] = *b"0123456789ABCDEF";

#[derive(Debug, Clone, thiserror::Error)]
/// Error produced by a communication channel
pub enum SlCanError {
    /// IO Error
    #[error("IO error")]
    IOError(#[from] #[source] Arc<std::io::Error>),
    /// Operation failed
    #[error("Operation failed")]
    OperationFailed,
    /// Unsupported speed
    #[error("Unsupported speed")]
    UnsupportedSpeed,
    /// Read timeout
    #[error("Read timeout")]
    ReadTimeout,
    /// Rx buffer full
    #[error("Rx buffer full")]
    RxBufferFull,
    /// Decoding failed
    #[error("Decoding failed")]
    DecodingFailed,
    /// Not acknowledged
    #[error("Not acknowledged")]
    NotAcknowledged,
}

/// SLCAN Result
pub type SlCanResult<T> = Result<T, SlCanError>;

const SLCAN_CAPABILITIES: HardwareCapabilities = HardwareCapabilities {
    iso_tp: true,
    can: true,
    ip: false,
    sae_j1850: false,
    kline: false,
    kline_kwp: false,
    sci: false,
};


/// SLCAN Device
#[derive(Clone)]
pub struct SlCanDevice {
    pub(crate) info: HardwareInfo,
    port: Arc<Mutex<Box<dyn SerialPort>>>,
    pub(crate) canbus_active: Arc<AtomicBool>,
    pub(crate) isotp_active: Arc<AtomicBool>,
    rx_queue: VecDeque<CanFrame>,
    rx_queue_limit: usize,
}

impl Debug for SlCanDevice {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "SlCanDevice {}", self.info.name)
    }
}
enum ReadWithAckResult {
    Ack,
    CanFrame(CanFrame)
}

impl SlCanDevice {
    /// Creates a new SLCAN device
    pub fn new(port: Box<dyn SerialPort>, rx_queue_limit: usize) -> Self {
        SlCanDevice {
            info: HardwareInfo {
                name: "slcan".into(),  // TODO: Get version and serial number from device
                vendor: None,
                capabilities: SLCAN_CAPABILITIES,
                device_fw_version: None,
                api_version: None,
                library_version: None,
                library_location: None,
            },
            port: Arc::new(Mutex::new(port)),
            canbus_active: Arc::new(AtomicBool::new(false)),
            isotp_active: Arc::new(AtomicBool::new(false)),
            rx_queue: VecDeque::new(),
            rx_queue_limit
        }
    }

    fn read_ack_or_packet(&mut self) -> SlCanResult<ReadWithAckResult> {
        let mut buf_1 = [0; 1];
        let mut buf = Vec::with_capacity(MAX_PACKET_SIZE);
        while self.port.lock().unwrap().read(&mut buf_1).map_err(convert_io_error)? == 1 {
            let byte = buf_1[0];
            if byte != b'\r' && byte != 0x7 {
                if buf.len() == MAX_PACKET_SIZE {
                    return Err(SlCanError::RxBufferFull);
                }
                buf.push(byte);
            } else {
                if buf.len() == 0 {
                    if buf_1[0] == 0x7 {
                        return Err(SlCanError::NotAcknowledged);
                    } else if buf_1[0] == b'\r' {
                        return Ok(ReadWithAckResult::Ack);
                    } else {
                        return Err(SlCanError::OperationFailed);
                    }
                }
                if buf.len() < 5 {
                    return Err(SlCanError::DecodingFailed);
                }
                match buf[0] {
                    b't' => {
                        let id = (hex_to_byte(buf[1])? as u32) << 8
                            | (hex_to_byte(buf[2])? as u32) << 4
                            | hex_to_byte(buf[3])? as u32;
                        let dlc = hex_to_byte(buf[4])?;

                        if dlc > 8 || buf.len() < (dlc * 2) as usize + 5 {
                            return Err(SlCanError::DecodingFailed);
                        }
                        let mut data = [0u8; 8];
                        for i in 0..dlc as usize {
                            data[i] = hex_to_byte(buf[5 + i * 2])? << 4
                                | hex_to_byte(buf[5 + i * 2 + 1])?;
                        }
                        return Ok(ReadWithAckResult::CanFrame(CanFrame::new(id, &data, false)));
                    },
                    b'T' => {
                        let id = (hex_to_byte(buf[1])? as u32) << 28
                            | (hex_to_byte(buf[2])? as u32) << 24
                            | (hex_to_byte(buf[3])? as u32) << 20
                            | (hex_to_byte(buf[4])? as u32) << 16
                            | (hex_to_byte(buf[5])? as u32) << 12
                            | (hex_to_byte(buf[6])? as u32) << 8
                            | (hex_to_byte(buf[7])? as u32) << 4
                            | hex_to_byte(buf[8])? as u32;
                        let dlc = hex_to_byte(buf[9])?;
                        if dlc > 8 || buf.len() <= (dlc * 2) as usize + 5 {
                            return Err(SlCanError::DecodingFailed);
                        }
                        let mut data = [0u8; 8];
                        for i in 0..dlc as usize {
                            data[i] = hex_to_byte(buf[10 + i * 2 + 1])? << 4
                                | hex_to_byte(buf[10 + i * 2])?;
                        }
                        return Ok(ReadWithAckResult::CanFrame(CanFrame::new(id, &data, true)));
                    },
                    _ => return Err(SlCanError::DecodingFailed)
                }
            }
        }
        Err(SlCanError::ReadTimeout)
    }

    fn send_command_with_ack(&mut self, cmd: &[u8]) -> SlCanResult<()> {
        self.port
            .lock()
            .unwrap()
            .write(cmd)
            .map_err(convert_io_error)?;
        // Trying to get ACK, but there can be other packet coming at that point
        // That packet is saved in queue and will be read with read later
        // Timeout of 1 second must be enough in order to get the ACK among other packets
        let instant = Instant::now();
        while instant.elapsed().as_millis() <= 1000 {
            match self.read_ack_or_packet()? {
                ReadWithAckResult::CanFrame(f) => {
                    if self.rx_queue.len() >= self.rx_queue_limit {
                        return Err(SlCanError::RxBufferFull);
                    }
                    self.rx_queue.push_back(f)
                },
                ReadWithAckResult::Ack => return Ok(())
            }
        }
        Err(SlCanError::ReadTimeout)
    }

    /// Sets can speed and opens SLCAN channel
    pub fn open(&mut self, can_speed: u32) -> SlCanResult<()> {
        self.send_command_with_ack(get_speed_cmd(can_speed)?.as_ref())?;
        self.send_command_with_ack(b"O\r")
    }

    /// Closes SLCAN channel
    pub fn close(&mut self) -> SlCanResult<()> {
        self.send_command_with_ack(b"C\r")
    }

    /// Reads can frames from SLCAN device
    pub fn read(&mut self) -> SlCanResult<CanFrame> {
        if let Some(f) = self.rx_queue.pop_front() {
            Ok(f)
        } else {
            match self.read_ack_or_packet()? {
                ReadWithAckResult::CanFrame(f) => Ok(f),
                ReadWithAckResult::Ack => Err(SlCanError::DecodingFailed),
            }
        }
    }

    /// Sends can frames to SLCAN device
    pub fn write(&mut self, frame: CanFrame) -> SlCanResult<()> {
        let mut buf = Vec::with_capacity(27);
        if frame.is_extended() {
            buf.push(b'T');
            let id = frame.get_address().to_be_bytes();
            for i in id {
                buf.push(HEX[i as usize >> 4]);
                buf.push(HEX[i as usize & 0xF]);
            }
        } else {
            buf.push(b't');
            let id = frame.get_address() & 0xFFF;
            buf.push(HEX[id as usize >> 8]);
            buf.push(HEX[(id as usize >> 4) & 0xF]);
            buf.push(HEX[id as usize & 0xF]);
        }
        buf.push(HEX[frame.get_data().len() & 0xF]);
        for d in frame.get_data() {
            buf.push(HEX[*d as usize >> 4]);
            buf.push(HEX[*d as usize & 0xF]);
        }
        buf.push(b'\r');
        self.send_command_with_ack(buf.as_slice())
    }

    /// Clears RX queue
    pub fn clear_rx_queue(&mut self) {
        self.rx_queue.clear();
    }
}

fn get_speed_cmd(can_speed: u32) -> SlCanResult<[u8; 3]> {
    match can_speed {
        10_000 => Ok(*b"S0\r"),
        20_000 => Ok(*b"S1\r"),
        50_000 => Ok(*b"S2\r"),
        100_000 => Ok(*b"S3\r"),
        125_000 => Ok(*b"S4\r"),
        250_000 => Ok(*b"S5\r"),
        500_000 => Ok(*b"S6\r"),
        800_000 => Ok(*b"S7\r"),
        1_000_000 => Ok(*b"S8\r"),
        83_333 => Ok(*b"S9\r"),  // Not supported by original standard
        _ => Err(SlCanError::UnsupportedSpeed)
    }
}

fn convert_io_error(error: std::io::Error) -> SlCanError {
    SlCanError::IOError(Arc::new(error))
}

fn hex_to_byte(hex: u8) -> SlCanResult<u8> {
    match hex {
        b'0'..=b'9' => Ok(hex - b'0'),
        b'a'..=b'f' => Ok(hex - b'a' + 10),
        b'A'..=b'F' => Ok(hex - b'A' + 10),
        _ => Err(SlCanError::DecodingFailed)
    }
}

impl From<SlCanError> for ChannelError {
    fn from(value: SlCanError) -> Self {
        match value {
            SlCanError::IOError(err) => ChannelError::IOError(err),
            SlCanError::ReadTimeout => ChannelError::ReadTimeout,
        _ => ChannelError::Other(value.to_string()),
        }
    }
}
