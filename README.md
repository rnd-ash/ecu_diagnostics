# ecu_diagnostics

[![Documentation](https://docs.rs/ecu_diagnostics/badge.svg)](https://docs.rs/ecu_diagnostics/)
[![Crates.io](https://img.shields.io/crates/v/ecu_diagnostics.svg)](https://crates.io/crates/ecu_diagnostics)
[![License](https://img.shields.io/crates/l/ecu_diagnostics.svg)](https://github.com/rnd-ash/ecu_diagnostics/blob/master/LICENSE)

A cross-platform crate with FFI bindings to allow for complex vehicle ECU diagnostics.

## IMPORTANT

This crate is a work in progress, and ECU compatibility may vary! This crate goes by the KWP2000 and UDS specification, but some ECUs choose to deviate slightly from the official specification!

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

## Diagnostic API features

The following features are working for each diagnostic server

### Global

These features are implemented for all diagnostic server

* SendByteArray
* SendByteArrayWithResponse

### OBD2

* Service01 - Live PIDs
* Service09 - Vehicle information
* Service04 - Clear DTCs


### KWP2000

Working specification services:

* ClearDiagnosticInformation
* ECUReset
* InputOutputControlByLocalIdentifier
* Enable/Disable messageTransmission
* ReadDataByIdentifier
* ReadDataByLocalIdentifier
* ReadECUIdentification (Specific for Daimler ECUs)
* ReadMemoryByAddress
* ReadStatusOfDTCs
* Routine control (Start, stop, status)
* SecurityAccess
* StartDiagnosticSession

### UDS

Working specification services:

* AccessTimingParameters
* ClearDiagnosticInformation
* CommunicationControl
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

* CAN
* ISO-TP

### D-PDU (ISO 22900-2)

Work in progress!!


# Notable contributions

* @LLBlumire
