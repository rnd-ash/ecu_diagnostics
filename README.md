# ecu_diagnostics

[![crates.io version](https://img.shields.io/crates/v/ecu_diagnostics.svg)](https://crates.io/crates/ecu_diagnostics)
[![docs.rs docs](https://docs.rs/ecu_diagnostics/badge.svg)](https://docs.rs/ecu_diagnostics)

A cross-platform crate for the diagnostic servers used for ECU diagnostics.


## Ensure you are running Rust 1.56.0 (2021 edition) or higher to use this crate!

## Features
* Easy to use (Check the examples folder)
* Implements UDS, KWP2000 and OBD2
* Hardware API for accessing common OBD-2 adapter types (Passthru)
* FFI bindings for use in C/C++ projects! (Check the examples folder)
* Safe to use (Cannot inadvertently send incorrect requests to the ECU)
* Parsing support - Where possible, data is returned in data structures, being interpreted from the ECU's response, rather than just bytes which have to be manually interpreted
* ISO-TP transport layer, LIN, J1850 and DoIP is work in progress at this time
* Diagnostic servers (For KWP2000 and UDS) automatically handle disconnects from ECU
* Optional diagnostic server event receiving for logging internal server events

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

# NEW (as of v0.91 UNIFIED DIAGNOSTIC SERVER)
The individual diagnostic servers are now merged into 1 diagnostic server that can handle all the different protocols
(Diagnostic protocol is specified at the servers creation). This dramatically reduces the crates bloat (Less copy/paste code),
and the refactoring has also introduced some new features:

* It is also possible now to define your own ECU Diagnostic protocol and session modes. Check the [examples](examples/) folder for how to do this!
* You can now set a hook function for when the ECU has received the request and is processing (Useful for longer running operations)
* You can now set a hook function for when transmit is completed and the server is waiting for the ECUs reply 
* The diagnostic server can now inform you if the ECU is connected or has been disconnected
* The diagnostic server can now automtically change the ECUs diagnostic session mode after a reboot to avoid 'ServiceNotSupportedInActiveSession'

## Diagnostic server checklist

### OBD2

Custom service support: YES

Working specification services:
* Service 01 - Show current data 
* Service 02 - Show data of freeze frame
* Service 09 - Request vehicle information

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


## Hardware API checklist

The Hardware API contains a common interface for scanning for compatible devices on a system as well as an API
for creating Channels for diagnostic servers using the hardware

### Passthru (SAE J2534)
* ISO-TP
* CAN
* Read Battery voltage

### SocketCAN
* ISO-TP
* CAN

### D-PDU (ISO 22900-2)
TBA


# Notable contributions
* @LLBlumire
* @nyurik (Created the [automotive_diag](https://github.com/nyurik/automotive_diag) crate, which this project now relies on)
