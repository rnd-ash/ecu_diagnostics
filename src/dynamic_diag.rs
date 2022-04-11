//! Dynamic diagnostic session helper
//!

use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

use crate::{
    channel::IsoTPSettings,
    dtc::DTC,
    hardware::Hardware,
    kwp2000::{self, Kwp2000DiagnosticServer, Kwp2000ServerOptions, Kwp2000VoidHandler},
    uds::{UDSSessionType, UdsDiagnosticServer, UdsServerOptions, UdsVoidHandler},
    DiagError, DiagServerResult, DiagnosticServer,
};

/// Dynamic diagnostic session
///
/// This is used if a target ECU has an unknown diagnostic protocol.
///
/// This also contains some useful wrappers for basic functions such as
/// reading and clearing error codes.
#[derive(Debug)]
pub struct DynamicDiagSession {
    session: DynamicSessionType,
}

#[derive(Debug)]
enum DynamicSessionType {
    Kwp(Kwp2000DiagnosticServer),
    Uds(UdsDiagnosticServer),
}

impl DynamicDiagSession {
    /// Creates a new dynamic session.
    /// This will first try with KWP2000, then if that fails,
    /// will try with UDS. If both server creations fail,
    /// then the last error will be returned.
    ///
    /// NOTE: In order to test if the ECU supports the protocol,
    /// the ECU will be put into extended diagnostic session briefly to test
    /// if it supports the tested diagnostic protocol.
    #[allow(unused_must_use, unused_assignments)]
    pub fn new_over_iso_tp<C>(
        hw_device: Arc<Mutex<C>>,
        channel_cfg: IsoTPSettings,
        tx_id: u32,
        rx_id: u32,
    ) -> DiagServerResult<Self>
    where
        C: Hardware + 'static,
    {
        let mut last_err: Option<DiagError>; // Setting up last recorded error

        // Create iso tp channel using provided HW interface. If this fails, we cannot setup KWP or UDS session!
        let mut iso_tp_channel = Hardware::create_iso_tp_channel(hw_device.clone())?;

        // Firstly, try KWP2000
        match Kwp2000DiagnosticServer::new_over_iso_tp(
            Kwp2000ServerOptions {
                send_id: tx_id,
                recv_id: rx_id,
                read_timeout_ms: 1500,
                write_timeout_ms: 1500,
                global_tp_id: 0x00,
                tester_present_interval_ms: 2000,
                tester_present_require_response: true,
                global_session_control: false
            },
            iso_tp_channel,
            channel_cfg,
            Kwp2000VoidHandler {},
        ) {
            Ok(mut kwp) => {
                if kwp
                    .set_diagnostic_session_mode(kwp2000::SessionType::ExtendedDiagnostics)
                    .is_ok()
                {
                    // KWP accepted! The ECU supports KWP2000!
                    // Return the ECU back to normal mode
                    kwp.set_diagnostic_session_mode(kwp2000::SessionType::Normal);
                    return Ok(Self {
                        session: DynamicSessionType::Kwp(kwp),
                    });
                } else {
                    last_err = Some(DiagError::NotSupported)
                }
            }
            Err(e) => {
                last_err = Some(e);
            }
        }

        iso_tp_channel = Hardware::create_iso_tp_channel(hw_device)?;
        match UdsDiagnosticServer::new_over_iso_tp(
            UdsServerOptions {
                send_id: tx_id,
                recv_id: rx_id,
                read_timeout_ms: 1500,
                write_timeout_ms: 1500,
                global_tp_id: 0x00,
                tester_present_interval_ms: 2000,
                tester_present_require_response: true,
            },
            iso_tp_channel,
            channel_cfg,
            UdsVoidHandler {},
        ) {
            Ok(mut uds) => {
                if uds.set_session_mode(UDSSessionType::Extended).is_ok() {
                    // UDS accepted! The ECU supports UDS!
                    // Return the ECU back to normal mode
                    uds.set_session_mode(UDSSessionType::Default);
                    return Ok(Self {
                        session: DynamicSessionType::Uds(uds),
                    });
                } else {
                    last_err = Some(DiagError::NotSupported)
                }
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
        Err(last_err.unwrap())
    }

    /// Returns a reference to KWP2000 session. None is returned if server type is not KWP2000
    pub fn as_kwp_session(&'_ mut self) -> Option<&'_ mut Kwp2000DiagnosticServer> {
        if let DynamicSessionType::Kwp(kwp) = self.session.borrow_mut() {
            Some(kwp)
        } else {
            None
        }
    }

    /// Performs operation with Kwp 2000 diagnostic server.
    /// If the type of the server is not KWP2000, then nothing happens, and DiagError::NotSupported
    pub fn with_kwp<T, F: Fn(&mut Kwp2000DiagnosticServer) -> DiagServerResult<T>>(
        &'_ mut self,
        f: F,
    ) -> DiagServerResult<T> {
        if let DynamicSessionType::Kwp(kwp) = self.session.borrow_mut() {
            f(kwp)
        } else {
            Err(DiagError::NotSupported)
        }
    }

    /// Performs operation with UDS diagnostic server.
    /// If the type of the server is not UDS, then nothing happens, and DiagError::NotSupported
    pub fn with_uds<T, F: Fn(&mut UdsDiagnosticServer) -> DiagServerResult<T>>(
        &'_ mut self,
        f: F,
    ) -> DiagServerResult<T> {
        if let DynamicSessionType::Uds(uds) = self.session.borrow_mut() {
            f(uds)
        } else {
            Err(DiagError::NotSupported)
        }
    }

    /// Returns a reference to UDS session. None is returned if server type is not UDS
    pub fn as_uds_session(&'_ mut self) -> Option<&'_ mut UdsDiagnosticServer> {
        if let DynamicSessionType::Uds(uds) = self.session.borrow_mut() {
            Some(uds)
        } else {
            None
        }
    }

    /// Puts the ECU into an extended diagnostic session
    pub fn enter_extended_diagnostic_mode(&mut self) -> DiagServerResult<()> {
        match self.session.borrow_mut() {
            DynamicSessionType::Kwp(k) => {
                k.set_diagnostic_session_mode(kwp2000::SessionType::ExtendedDiagnostics)
            }
            DynamicSessionType::Uds(u) => u.set_session_mode(UDSSessionType::Extended),
        }
    }

    /// Puts the ECU into a default diagnostic session. This is how the ECU normally operates
    pub fn enter_default_diagnostic_mode(&mut self) -> DiagServerResult<()> {
        match self.session.borrow_mut() {
            DynamicSessionType::Kwp(k) => {
                k.set_diagnostic_session_mode(kwp2000::SessionType::Normal)
            }
            DynamicSessionType::Uds(u) => u.set_session_mode(UDSSessionType::Default),
        }
    }

    /// Reads all diagnostic trouble codes from the ECU
    pub fn read_all_dtcs(&mut self) -> DiagServerResult<Vec<DTC>> {
        match self.session.borrow_mut() {
            DynamicSessionType::Kwp(k) => k.read_stored_dtcs(kwp2000::DTCRange::All),
            DynamicSessionType::Uds(u) => u.get_dtcs_by_status_mask(0xFF),
        }
    }

    /// Attempts to clear all DTCs stored on the ECU
    pub fn clear_all_dtcs(&mut self) -> DiagServerResult<()> {
        match self.session.borrow_mut() {
            DynamicSessionType::Kwp(k) => k.clear_dtc_range(kwp2000::ClearDTCRange::AllDTCs),
            DynamicSessionType::Uds(u) => u.clear_diagnostic_information(0x00FFFFFF),
        }
    }

    /// Attempts to send a payload of bytes to the ECU, and return its full response
    pub fn send_bytes_with_response(&mut self, payload: &[u8]) -> DiagServerResult<Vec<u8>> {
        match self.session.borrow_mut() {
            DynamicSessionType::Kwp(k) => k.send_byte_array_with_response(payload),
            DynamicSessionType::Uds(u) => u.send_byte_array_with_response(payload),
        }
    }

    /// Attempts to send a payload of bytes to the ECU, and don't poll for a response
    pub fn send_bytes(&mut self, payload: &[u8]) -> DiagServerResult<()> {
        match self.session.borrow_mut() {
            DynamicSessionType::Kwp(k) => k.send_byte_array(payload),
            DynamicSessionType::Uds(u) => u.send_byte_array(payload),
        }
    }
}

impl From<Kwp2000DiagnosticServer> for DynamicDiagSession {
    fn from(s: Kwp2000DiagnosticServer) -> Self {
        Self {
            session: DynamicSessionType::Kwp(s),
        }
    }
}

impl From<UdsDiagnosticServer> for DynamicDiagSession {
    fn from(s: UdsDiagnosticServer) -> Self {
        Self {
            session: DynamicSessionType::Uds(s),
        }
    }
}
