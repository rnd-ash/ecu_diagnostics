use libloading::Library;
use winapi::shared::minwindef::{WORD, DWORD};
use winapi::um::winnt::LPSTR;
use std::ffi::c_void;
use std::mem::size_of;
use std::path::Path;
use std::sync::Arc;
use std::fmt;

use crate::channel::{CanFrame, ChannelResult, ChannelError, Packet};
use crate::hardware::pcan_usb::pcan_types::PCANParameter;
use crate::hardware::{HardwareResult, HardwareError};

use super::pcan_types::{TpCanTimestamp, TpCanMsg, TpCanMsgFD, PCanResult, PCANError, PCanErrorTy, PcanUSB, PCANBaud, MsgType};


//pub type PassthruResult<T> = Result<T, PCANError>;

type GetErrorTextFn = unsafe extern "system" fn(
    error: i32,
    languge: u8,
    buffer: LPSTR
) -> i32;

type GetStatusFn = unsafe extern "system" fn(
    channel: WORD
) -> i32;

type InitializeFn = unsafe extern "system" fn(
    channel: WORD, 
    btr0btr1: WORD) -> i32;

type InitializeFdFn = unsafe extern "system" fn(
    channel: WORD,
    bitrate: LPSTR
) -> i32;

type LookUpChannelFn = unsafe extern "system" fn(
    paramters: LPSTR,
    found_channel: *mut DWORD
) -> i32;

type ReadFn = unsafe extern "system" fn(
    channel: WORD,
    buffer: *mut TpCanMsg,
    timestamp: *mut TpCanTimestamp
) -> i32;

type ReadFdFn = unsafe extern "system" fn(
    channel: WORD,
    buffer: *mut TpCanMsgFD,
    timestamp: *mut TpCanTimestamp
) -> i32;

type ResetFn = unsafe extern "system" fn(
    channel: WORD
) -> i32;

type FilterMessagesFn = unsafe extern "system" fn(
    channel: WORD,
    from_id: DWORD,
    to_id: DWORD,
    mode: u8
) -> i32;

type GetValueFn = unsafe extern "system" fn(
    channel: WORD,
    parameter: DWORD,
    buffer: *mut c_void,
    buffer_len: DWORD
) -> i32;

type SetValueFn = unsafe extern "system" fn(
    channel: WORD,
    parameter: DWORD,
    buffer: *mut c_void,
    buffer_len: DWORD
) -> i32;

type UninitalizeFn = unsafe extern "system" fn(
    channel: WORD,
) -> i32;

type WriteFn = unsafe extern "system" fn(
    channel: WORD,
    buffer: *mut TpCanMsg,
) -> i32;

type WriteFdFn = unsafe extern "system" fn(
    channel: WORD,
    buffer: *mut TpCanMsgFD,
) -> i32;


fn check_pcan_func_result<T>(ret: T, status: i32) -> PCanResult<T> {
    match status {
        0 => PCanResult::Ok(ret),
        x => {
            if let Some(r) = PCANError::from_repr(x) {
                PCanResult::Err(PCanErrorTy::StandardError(r))
            } else {
                log::error!("PCAN Unknown Drv error {x}");
                PCanResult::Err(PCanErrorTy::Unknown(x))
            }
        }
    }    
}

#[derive(Clone)]
pub struct PCanDrv {
    path: &'static str,
    /// Loaded library to interface with the device
    lib: Arc<Library>,
    /// Is the device currently connected?
    is_connected: bool,
    get_error_text_fn: GetErrorTextFn,
    get_status_fn: GetStatusFn,
    initialize_fn: InitializeFn,
    initialize_fd_fn: InitializeFdFn,
    lookup_channel_fn: LookUpChannelFn,
    read_fn: ReadFn,
    read_fd_fn: ReadFdFn,
    reset_fn: ResetFn,
    filter_messages_fn: FilterMessagesFn,
    get_value_fn: GetValueFn,
    set_value_fn: SetValueFn,
    uninitialize_fn: UninitalizeFn,
    write_fn: WriteFn,
    write_fd_fn: WriteFdFn,
    
}

impl fmt::Debug for PCanDrv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PcanDrv")
            .field("is_connected", &self.is_connected)
            .field("library", &self.lib)
            .finish()
    }
}

impl PCanDrv {
    pub fn load_lib() -> HardwareResult<PCanDrv> {
        let path: &'static str = if cfg!(target_pointer_width="32") {
            match Path::new("C:\\Program Files (x86)\\").exists() {
                true => "C:\\Windows\\SysWOW64\\PCANBasic.dll", // 64bit
                false => "C:\\Windows\\System32\\PCANBasic.dll", // Native 32bit
            }
        } else {
            "C:\\Windows\\System32\\PCANBasic.dll"
        };
        log::debug!("Opening function library {path}");
        let lib = unsafe { Library::new(path)? };
        let res = unsafe {
            Self {
                path,
                get_error_text_fn: *lib.get::<GetErrorTextFn>(b"CAN_GetErrorText\0")?.into_raw(),
                get_status_fn: *lib.get::<GetStatusFn>(b"CAN_GetStatus\0")?.into_raw(),
                initialize_fn: *lib.get::<InitializeFn>(b"CAN_Initialize\0")?.into_raw(),
                initialize_fd_fn: *lib.get::<InitializeFdFn>(b"CAN_InitializeFD\0")?.into_raw(),
                lookup_channel_fn: *lib.get::<LookUpChannelFn>(b"CAN_LookUpChannel\0")?.into_raw(),
                read_fn: *lib.get::<ReadFn>(b"CAN_Read\0")?.into_raw(),
                read_fd_fn: *lib.get::<ReadFdFn>(b"CAN_ReadFD\0")?.into_raw(),
                reset_fn: *lib.get::<ResetFn>(b"CAN_Reset\0")?.into_raw(),
                filter_messages_fn: *lib.get::<FilterMessagesFn>(b"CAN_FilterMessages\0")?.into_raw(),
                get_value_fn: *lib.get::<GetValueFn>(b"CAN_GetValue\0")?.into_raw(),
                set_value_fn: *lib.get::<SetValueFn>(b"CAN_SetValue\0")?.into_raw(),
                uninitialize_fn: *lib.get::<UninitalizeFn>(b"CAN_Uninitialize\0")?.into_raw(),
                write_fn: *lib.get::<WriteFn>(b"CAN_Write\0")?.into_raw(),
                write_fd_fn: *lib.get::<WriteFdFn>(b"CAN_WriteFD\0")?.into_raw(),
                lib: Arc::new(lib),
                is_connected: false,
            }
        };
        res.reset_driver()?;
        Ok(res)
    }

    fn reset_driver(&self) -> HardwareResult<()> {
        log::debug!("PCAN Reset called");
        let res = unsafe { (self.uninitialize_fn)(0x00) };
        check_pcan_func_result((), res).map_err(|e| e.into())
    }

    pub (crate) fn reset_handle(&self, handle: u16) -> HardwareResult<()> {
        log::debug!("PCAN Reset Handle called: {handle}");
        let res = unsafe { (self.reset_fn)(handle) };
        let e = check_pcan_func_result((), res);
        if let Err(PCanErrorTy::StandardError(PCANError::Initialize)) = e {
            // This is actually OK
            return Ok(())
        }
        e.map_err(|e| e.into())
    }

    pub (crate) fn get_device_info(&self, handle: &PcanUSB) -> HardwareResult<(String, String)> {
        let mut n: [u8; 33] = [0; 33];
        let mut v: [u8; 256] = [0; 256];

        check_pcan_func_result(
            (), 
            unsafe { (self.get_value_fn)(*handle as u16, PCANParameter::HardwareName as u32, &mut n as *mut _ as *mut c_void, 256) }
        ).map_err(|e| HardwareError::from(e))?;

        check_pcan_func_result(
            (), 
            unsafe { (self.get_value_fn)(*handle as u16, PCANParameter::APIVersion as u32, &mut v as *mut _ as *mut c_void, 33) }
        ).map_err(|e| HardwareError::from(e))?;

        let name = String::from_utf8(n.to_vec()).unwrap();
        let version = String::from_utf8(v.to_vec()).unwrap();
        Ok((name, version))
    }

    pub (crate) fn get_path(&self) -> &'static str {
        self.path
    }

    pub (crate) fn initialize_can(&mut self, handle: PcanUSB, baud: PCANBaud) -> HardwareResult<()> {
        log::debug!("Initialize can called. Baud: {baud:?}");
        check_pcan_func_result(
            (),
            unsafe { (self.initialize_fn)(handle as u16, baud as u16) }
        ).map_err(|e| HardwareError::from(e))?;
    
        let mut param: u8 = 0x01;
        let mut p_type = PCANParameter::MessageFilter as u32;

        check_pcan_func_result(
            (),
            unsafe { (self.set_value_fn)(handle as u16, p_type, &mut param as *mut _ as *mut c_void , 1) }
        ).map_err(|e| HardwareError::from(e))?;

        p_type = PCANParameter::BusOffAutoReset as u32;

        check_pcan_func_result(
            (),
            unsafe { (self.set_value_fn)(handle as u16, p_type, &mut param as *mut _ as *mut c_void , 1) }
        ).map_err(|e| HardwareError::from(e))
    }

    pub (crate) fn read(&mut self, handle: PcanUSB) -> ChannelResult<CanFrame> {
        let mut can_msg = TpCanMsg {
            id: 0,
            msg_type: MsgType::Standard,
            len: 0,
            data: [0; 8],
        };
        let res = unsafe { (self.read_fn)(handle as u16, &mut can_msg, std::ptr::null_mut()) };
        check_pcan_func_result(
            (),
            res
        ).map_err(|e| {
            if e == PCanErrorTy::StandardError(PCANError::QrcvEmpty) {
                ChannelError::BufferEmpty
            } else {
                HardwareError::from(e).into()
            }
        })?;
        // Read OK!
        Ok(CanFrame::new(can_msg.id, &can_msg.data[0..can_msg.len as usize], can_msg.msg_type == MsgType::Extended))
    }

    pub (crate) fn write(&mut self, handle: PcanUSB, packet: CanFrame) -> ChannelResult<()> {
        let mut can_msg = TpCanMsg {
            id: packet.get_address(),
            msg_type: if packet.is_extended() { MsgType::Extended } else { MsgType::Standard },
            len: packet.get_data().len() as u8,
            data: [0; 8],
        };
        let l = packet.get_data().len();
        can_msg.data[0..l].copy_from_slice(packet.get_data());
        check_pcan_func_result(
            (),
        unsafe { (self.write_fn)(handle as u16, &mut can_msg) }
        ).map_err(|e| HardwareError::from(e))?;
        Ok(())
    }
}
