//! The passthru API (Also known as SAE J2534) is an adapter protocol used by some OBD2 adapters.
//!
//! This module provides support for V04.04 of the API, including experimental support for OSX and Linux, used by
//! [Macchina-J2534](http://github.com/rnd-ash/macchina-J2534)
//!
//! The API supports the following communication protocols:
//! * ISO9141 
//! * ISO15475
//! * ISO14230-4
//! * J1850 PWM
//! * J1850 VPW
//! * SCI
//! * CAN
// however it should be noted that adapters might only support a range of these protocols

