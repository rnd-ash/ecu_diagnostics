//! Provides methods to manipulate the ECUs diagnostic session mode

/// KWP2000 diagnostic session type
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kwp2000SessionType {
    /// Normal session. The ECU will typically boot in this state.
    /// In this mode, only non-intrusive functions are supported
    Normal,
    /// Reprogramming session. Used for flashing an ECU. Only functions
    /// for reading/writing to memory are allowed in this mode
    Reprogramming,
    /// In stand-by mode, the ECU will be in a low-power state,
    /// acting as a slave to other ECUs and only able to perform actuation tests
    /// at the request of a tester. If a request is made to the ECU which can disrupt
    /// its low power state, the ECU will reject the request.
    Standby,
    /// In this mode, the ECU will remain active, but will disable
    /// all normal CAN/LIN communication with the vehicle, effectively putting
    /// the ECU to sleep. IMPORTANT. If the ECU is power cycled, it will
    /// reboot in this mode.
    ///
    /// This mode is only intended for ECU development, so it will likely not
    /// be available on production ECUs.
    Passive,
    /// Extended diagnostics mode. Every service is available here
    ExtendedDiagnostics,
    /// Custom diagnostic mode not in the KWP2000 specification
    Custom(u8),
}

impl From<Kwp2000SessionType> for u8 {
    fn from(x: Kwp2000SessionType) -> Self {
        match x {
            Kwp2000SessionType::Normal => 0x81,
            Kwp2000SessionType::Reprogramming => 0x85,
            Kwp2000SessionType::Standby => 0x89,
            Kwp2000SessionType::Passive => 0x90,
            Kwp2000SessionType::ExtendedDiagnostics => 0x92,
            Kwp2000SessionType::Custom(c) => c,
        }
    }
}
