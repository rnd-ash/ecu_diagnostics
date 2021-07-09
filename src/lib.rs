#![no_std]

pub mod kwp2000;
pub mod obd2;
pub mod uds;

extern crate alloc;
use alloc::{vec::Vec};


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagError {
    NotSupported,
    IOError,
    WriteError,
    ReadError,
    Timeout,
    ECUError(u8)
}

pub type DiagServerResult<T> = Result<T, DiagError>;


#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChannelConfig {
    /// ISO TP Separation time (Min)
    ISO_TP_ST_MIN(u8),
    /// ISO TP block size
    ISO_TP_BS(u8),
}


/// A dynamic trait which must be implemented in order to send
/// and receive bytes to/from an ECU

pub trait ECUCommChannel: Sized {
    fn configure_channel(&mut self, config_option: ChannelConfig) -> DiagServerResult<()>;
    fn write_bytes_to_ecu(&mut self, x: &[u8], timeout_ms: u32) -> DiagServerResult<()>;
    fn write_bytes_to_global_addr(&mut self, addr: u32, x: &[u8], timeout_ms: u32) -> DiagServerResult<()>;
    fn read_bytes_from_ecu(&mut self, timeout_ms: u32) -> DiagServerResult<Vec<u8>>;
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


/// Basic ECU Diagnostic server
/// This trait allows all 3 diagnostic servers (OBD/KWP/UDS) to be combined under a common trait
/// Where the ECU is kept in a read-only state of operation (No Write support).
/// 
/// Check the individual protocols for which functions are supported under the Basic server (KWP/UDS)
/// All OBD functions are supported under the basic diagnostic server
pub trait BasicECUDiagServer<T> where T: ECUCommChannel {
    fn start_server_canbus(&mut self, channel: T) -> DiagServerResult<()>;
    fn start_server_kline(&mut self, channel: T) -> DiagServerResult<()>;
    fn update_server_loop(&mut self);
    fn read_dtcs(&mut self) -> DiagServerResult<Vec<DTC>>;
    fn clear_dtcs(&mut self) -> DiagServerResult<()>;
    fn stop_server(&mut self);
}

/// Advanced ECU Diagnostic server
/// Only KWP and UDS implement this, and activating these servers puts the ECU in a non-default
/// diagnostic state, which can be dangerous
pub trait AdvancedECUDiagServer<T> : BasicECUDiagServer<T> where T: ECUCommChannel {
    type DiagnosticSessionModes : Into<u8>;
    type DiagnosticErrors: From<u8>;

    fn enter_session_mode(&mut self, mode: Self::DiagnosticSessionModes) -> DiagServerResult<()>;

    fn execute_custom_pid(&mut self, pid: u8, data: &[u8]) -> DiagServerResult<Vec<u8>>;
}