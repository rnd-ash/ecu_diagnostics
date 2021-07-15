# ecu_diagnostics

[![Documentation](https://docs.rs/ecu_diagnostics/badge.svg)](https://docs.rs/ecu_diagnostics/)
[![Crates.io](https://img.shields.io/crates/v/ecu_diagnostics.svg)](https://crates.io/crates/ecu_diagnostics)
[![License](https://img.shields.io/crates/l/ecu_diagnostics.svg)](https://github.com/rnd-ash/ecu_diagnostics/blob/master/LICENSE)

A cross-platform crate with FFI bindings to allow for complex vehicle ECU diagnostics.

## IMPORTANT
Currently this crate is not 100% ready for use. Check the feature matrix below for each diagnostic
server's checklist

## Features
* Easy to use (Check the examples folder for UDS server usage with SocketCAN)
* Implements UDS


## A quick overview of diagnostic servers used by ECUs

### On-board diagnostics (OBD2)
ISO9141 - OBD2 is a legal requirement on all vehicles produced from 2002, allowing for
reading of sensor data, reading and clearing standard DTCs, and reading basic vehicle information.
OBD2 is designed to be safe and simple, and does not write data to the ECU.



### Keyword protocol 2000 (KWP2000)
ISO14230 - KWP2000 is a advanced diagnostic protocol utilized by many vehicle manufacturers from 2000-2006 (Superseded by UDS).
Unlike OBD2, KWP2000 allows for much more complex operations, which could potentially cause damage to a vehicle if used incorrectly.  
 A few examples of features allowed by KWP2000 are
 * ECU flashing
 * Clearing and reading of permanent DTCs
 * Manipulation of ECU communication parameters
 * Low level manipulation of ECU's EEPROM or RAM
 * Gateway access in vehicles which have them

 The specification implemented in this crate is v2.2, dated 05-08-2002

 ### Unified diagnostic services (UDS)
 ISO14429 - UDS is an advanced diagnostic protocol utilized by almost all vehicle manufacturers from 2006 onwards. Like KWP2000,
 this protocol allows for reading/writing directly to the ECU, and should therefore be used with caution.

 The specification implemented in this crate is the second edition, dated 01-12-2006.



## Diagnostic server checklist

### OBD2
TBA

### KWP2000

Custom service support: YES

Working specification services:
* StartDiagnosticSession
* ECUReset
* ReadDTCByStatus
* ReadECUIdentification
* ReadStatusOfDTC
* ClearDiagnosticInformation

### UDS

Custom service support: YES

Working specification services:

* DiagnosticSessionControl
* ECUReset
* ReadDTCInformation
* SecurityAccess
