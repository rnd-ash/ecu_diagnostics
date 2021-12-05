//! Lib functions for D-PDU API
use libloading::Library;
use std::os::raw::c_char;
use std::sync::Arc;
use std::{ffi::*, fmt};

use super::c_types::{self, PDU_DATA_ITEM, T_PDU_ERROR};

type PDUConstructFn = unsafe extern "stdcall" fn(optionStr: *const char, pAPITag: *mut c_void) -> T_PDU_ERROR;
type PDUDestructFn = unsafe extern "stdcall" fn() -> T_PDU_ERROR;
type PDUModuleConnectFn = unsafe extern "stdcall" fn(hMod: u32) -> T_PDU_ERROR;
type PDUModuleDisconnectFn = unsafe extern "stdcall" fn(hMod: u32) -> T_PDU_ERROR;
type PDUGetTimeStampFn = unsafe extern "stdcall" fn(hMod: u32, pTimestamp: *mut u32) -> T_PDU_ERROR;
type PDUIoCtlFn = unsafe extern "stdcall" fn(hMod: u32, hCll: u32, ioctlCommandID: u32, pInputData: *mut PDU_DATA_ITEM, pOutputData: *mut PDU_DATA_ITEM) -> T_PDU_ERROR;