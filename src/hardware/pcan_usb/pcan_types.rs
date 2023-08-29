use enum_repr::EnumRepr;
use thiserror::Error;
use winapi::shared::minwindef::{DWORD, WORD};

use crate::{channel::ChannelError, hardware::HardwareError};

const MAX_LENGTH_HARDWARE_NAME: usize = 33;
const MAX_LENGTH_VERSION_STRING: usize = 256;

pub enum PcanEnumWrapper<T, E> {
    Std(T),
    Unknown(E),
}

pub(crate) const ALL_USB_DEVICES: &[PcanUSB] = &[
    PcanUSB::USB1,
    PcanUSB::USB2,
    PcanUSB::USB3,
    PcanUSB::USB4,
    PcanUSB::USB5,
    PcanUSB::USB6,
    PcanUSB::USB7,
    PcanUSB::USB8,
    PcanUSB::USB9,
    PcanUSB::USB10,
    PcanUSB::USB11,
    PcanUSB::USB12,
    PcanUSB::USB13,
    PcanUSB::USB14,
    PcanUSB::USB15,
    PcanUSB::USB16,
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[EnumRepr(type = "i16")]
pub enum PcanUSB {
    USB1 = 0x51,
    USB2 = 0x52,
    USB3 = 0x53,
    USB4 = 0x54,
    USB5 = 0x55,
    USB6 = 0x56,
    USB7 = 0x57,
    USB8 = 0x58,
    USB9 = 0x509,
    USB10 = 0x50A,
    USB11 = 0x50B,
    USB12 = 0x50C,
    USB13 = 0x50D,
    USB14 = 0x50E,
    USB15 = 0x50F,
    USB16 = 0x510,
}

pub type PcanUSBWrapper = PcanEnumWrapper<PcanUSB, u16>;

#[repr(u8)]
pub enum PcanMessageType {
    Standard = 0x00,
    Rtr = 0x01,
    Extended = 0x02,
    Fd = 0x04,
    Brs = 0x08,
    Esi = 0x10,
    Echo = 0x20,
    ErrFrame = 0x40,
    Status = 0x80,
}

pub type PcanMessageTypeWrapper = PcanEnumWrapper<PcanMessageType, u8>;

#[repr(u8)]
pub enum PcanServiceState {
    Stopped = 0x01,
    Running = 0x04,
}

pub type PcanServiceStateWrapper = PcanEnumWrapper<PcanServiceState, u8>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[EnumRepr(type = "u32")]
pub(crate) enum PCANParameter {
    DeviceID = 0x01,
    FiveVoltPower = 0x02,
    ReceiveEvent = 0x03,
    MessageFilter = 0x04,
    APIVersion = 0x05,
    ChannelVersion = 0x06,
    BusOffAutoReset = 0x07,
    ListenOnly = 0x08,
    LogLocation = 0x09,
    LogStatus = 0x0A,
    LogConfigure = 0x0B,
    LogText = 0x0C,
    ChannelCondition = 0x0D,
    HardwareName = 0x0E,
    ReceiveStatus = 0x0F,
    ControllerNumber = 0x10,
    TraceLocation = 0x11,
    TraceStatus = 0x12,
    TraceSize = 0x13,
    TraceConfigure = 0x14,
    ChannelIdentifying = 0x15,
    ChannelFeatures = 0x16,
    BitRateAdapting = 0x17,
    BitRateInfo = 0x18,
    BitRateInfoFD = 0x19,
    BusSpeedNominal = 0x1A,
    BusSpeedData = 0x1B,
    IpAddress = 0x1C,
    LanServiceStatus = 0x1D,
    AllowStatusFrames = 0x1E,
    AllowRTRFrames = 0x1F,
    AllowErrorFrames = 0x20,
    InterFrameDelay = 0x21,
    AcceptanceFilter11Bit = 0x22,
    AcceptanceFilter29Bit = 0x23,
    IoDigitalConfiguration = 0x24,
    IoDigitalValue = 0x25,
    IoDigitalSet = 0x26,
    IoDigitalClear = 0x27,
    IoAnalogValue = 0x28,
    FirmwareVersion = 0x29,
    AttachedChannelCount = 0x2A,
    AttachedChannels = 0x2B,
    AllowEchoFrames = 0x2C,
    DevicePartNumber = 0x2D,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[repr(C)]
pub enum PCANBaud {
    Can1Mbps = 0x0014,
    Can800Kbps = 0x0016,
    Can500Kbps = 0x001C,
    Can250Kbps = 0x011C,
    Can125Kbps = 0x031C,
    Can100Kbps = 0x432F,
    Can95Kbps = 0xC34E,
    Can83Kbps = 0x852B,
    Can50Kbps = 0x472F,
    Can47Kbps = 0x1414,
    Can33Kbps = 0x8B2F,
    Can20Kbps = 0x532F,
    Can10Kbps = 0x672F,
    Can5Kbps = 0x7F7F,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Error)]
#[EnumRepr(type = "i32")]
pub enum PCANError {
    #[error("Transmit buffer in CAN controller is full")]
    XMTFull = 0x00001,
    #[error("CAN controller was read too late")]
    Overrun = 0x00002,
    #[error("Bus error: an error counter reached the 'light' limit")]
    BusLight = 0x00004,
    #[error("Bus error: an error counter reached the 'heavy' limit")]
    BusHeavy = 0x00008,
    #[error("Bus error: the CAN controller is in bus-off state")]
    BusOff = 0x00010,
    #[error("Receive queue is empty")]
    QrcvEmpty = 0x00020,
    #[error("Receive queue was read too late")]
    Qoverrun = 0x00040,
    #[error("Transmit queue is full")]
    QxmtFull = 0x00080,
    #[error("Test of the CAN controller hardware registers failed (no hardware found)")]
    RegTest = 0x00100,
    #[error("Driver not loaded")]
    NoDriver = 0x00200,
    #[error("Hardware already in use by a Net")]
    HwInUse = 0x00400,
    #[error("A Client is already connected to the Net")]
    NetInUse = 0x00800,
    #[error("Hardware handle is invalid")]
    IllHw = 0x1400,
    #[error("Net handle is invalid")]
    IllNet = 0x1800,
    #[error("Client handle is invalid")]
    IllClient = 0x1C00,
    #[error("Resource cannot be created")]
    Resource = 0x02000,
    #[error("Invalid parameter")]
    IllParamType = 0x04000,
    #[error("Invalid parameter value")]
    IllParamVal = 0x08000,
    #[error("Unknown error")]
    Unknown = 0x010000,
    #[error("Invalid data, function or action")]
    IllData = 0x20000,
    #[error("Driver object state is wrong for the attempted operation")]
    IllMode = 0x80000,
    #[error("An operation was successfully carried out, however, irregularities were registered")]
    Caution = 0x2000000,
    #[error("Channel is not initialized")]
    Initialize = 0x4000000,
    #[error("Invalid operation")]
    IllOperation = 0x8000000,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[EnumRepr(type = "u8")]
pub enum MsgType {
    Standard = 0x00,
    Rtr = 0x01,
    Extended = 0x02,
    Fd = 0x04,
    Brs = 0x08,
    Esi = 0x10,
    Echo = 0x20,
    ErrFrae = 0x40,
    Status = 0x80,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Error)]
pub enum PCanErrorTy {
    #[error(transparent)]
    StandardError(#[from] PCANError),
    #[error("An unknown error code of 0x{0:08X?} was returned by the PCAN API")]
    Unknown(i32),
}

pub type PCanResult<T> = Result<T, PCanErrorTy>;

#[repr(C)]
#[derive(Debug)]
pub struct TpCanMsg {
    pub(crate) id: DWORD,
    pub(crate) msg_type: MsgType,
    pub(crate) len: u8,
    pub(crate) data: [u8; 8],
}

#[repr(C)]
pub struct TpCanTimestamp {
    pub(crate) millis: DWORD,
    pub(crate) millis_overflow: WORD,
    pub(crate) micros: WORD,
}

#[repr(C)]
pub struct TpCanMsgFD {
    pub(crate) id: DWORD,
    pub(crate) msg_type: MsgType,
    pub(crate) dlc: u8,
    pub(crate) data: [u8; 64],
}

#[repr(C)]
pub struct TpCanChannelInformation {
    channel_handle: WORD,
    device_type: u8,
    controller_number: u8,
    device_features: DWORD,
    device_name: [u8; MAX_LENGTH_HARDWARE_NAME],
    device_id: DWORD,
    channel_condition: DWORD,
}
