use j2534_rust::FilterType::FLOW_CONTROL_FILTER;
use j2534_rust::*;
use libloading::Library;
use std::os::raw::c_char;
use std::sync::Arc;
use std::{ffi::*, fmt};

/// Result which contains a PASSTHRU_ERROR in it's Err() variant
pub type PassthruResult<T> = Result<T, PassthruError>;

type PassThruOpenFn = unsafe extern "stdcall" fn(name: *const c_void, device_id: *mut u32) -> i32;
type PassThruCloseFn = unsafe extern "stdcall" fn(device_id: u32) -> i32;
type PassThruConnectFn = unsafe extern "stdcall" fn(
    device_id: u32,
    protocol_id: u32,
    flags: u32,
    baudrate: u32,
    channel_id: *mut u32,
) -> i32;
type PassThruDisconnectFn = unsafe extern "stdcall" fn(channel_id: u32) -> i32;
type PassThruReadMsgsFn = unsafe extern "stdcall" fn(
    channel_id: u32,
    msgs: *mut PASSTHRU_MSG,
    num_msgs: *mut u32,
    timeout: u32,
) -> i32;
type PassThruWriteMsgsFn = unsafe extern "stdcall" fn(
    channel_id: u32,
    msgs: *mut PASSTHRU_MSG,
    num_msgs: *mut u32,
    timeout: u32,
) -> i32;
type PassThruStartPeriodicMsgFn = unsafe extern "stdcall" fn(
    channel_id: u32,
    msg: *const PASSTHRU_MSG,
    msg_id: *mut u32,
    time_interval: u32,
) -> i32;
type PassThruStopPeriodicMsgFn = unsafe extern "stdcall" fn(channel_id: u32, msg_id: u32) -> i32;
type PassThruStartMsgFilterFn = unsafe extern "stdcall" fn(
    channel_id: u32,
    filter_type: u32,
    m_msg: *const PASSTHRU_MSG,
    p_msg: *const PASSTHRU_MSG,
    fc_msg: *const PASSTHRU_MSG,
    filter_id: *mut u32,
) -> i32;
type PassThruStopMsgFilterFn = unsafe extern "stdcall" fn(channel_id: u32, filter_id: u32) -> i32;
type PassThruSetProgrammingVoltageFn =
    unsafe extern "stdcall" fn(device_id: u32, pin_number: u32, voltage: u32) -> i32;
type PassThruReadVersionFn = unsafe extern "stdcall" fn(
    device_id: u32,
    firmware_version: *mut c_char,
    dll_version: *mut c_char,
    api_version: *mut c_char,
) -> i32;
type PassThruGetLastErrorFn = unsafe extern "stdcall" fn(error_description: *mut c_char) -> i32;
type PassThruIoctlFn = unsafe extern "stdcall" fn(
    handle_id: u32,
    ioctl_id: u32,
    input: *mut c_void,
    output: *mut c_void,
) -> i32;

#[derive(Debug)]
pub struct DrvVersion {
    /// Library (DLL) Version
    pub dll_version: String,
    /// Passthru API Version (Only V04.04 is supported currently!)
    pub api_version: String,
    /// Device Firmware version
    pub fw_version: String,
}

#[derive(Clone)]
pub struct PassthruDrv {
    /// Loaded library to interface with the device
    lib: Arc<Library>,
    /// Is the device currently connected?
    is_connected: bool,
    /// Open device connection
    open_fn: PassThruOpenFn,
    /// Close device connection
    close_fn: PassThruCloseFn,
    /// Connect a communication channel
    connect_fn: PassThruConnectFn,
    /// Disconnect a communication channel
    disconnect_fn: PassThruDisconnectFn,
    /// Read messages from a communication channel
    read_msg_fn: PassThruReadMsgsFn,
    /// Write messages to a communication channel
    write_msg_fn: PassThruWriteMsgsFn,
    /// Start a periodic message
    start_periodic_fn: PassThruStartPeriodicMsgFn,
    /// Stop a periodic message
    stop_periodic_fn: PassThruStopPeriodicMsgFn,
    /// Start a filter on a channel
    start_filter_fn: PassThruStartMsgFilterFn,
    /// Stop a filter on a channel
    stop_filter_fn: PassThruStopMsgFilterFn,
    /// Set programming voltage
    set_prog_v_fn: PassThruSetProgrammingVoltageFn,
    /// Get the last driver error description if ERR_FAILED
    get_last_err_fn: PassThruGetLastErrorFn,
    /// IOCTL
    ioctl_fn: PassThruIoctlFn,
    /// Get driver details
    read_version_fn: PassThruReadVersionFn,
}

impl fmt::Debug for PassthruDrv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PassthruDrv")
            .field("is_connected", &self.is_connected)
            .field("library", &self.lib)
            .finish()
    }
}

#[inline(always)]
/// Function to reduce boilerplate code with returning a Result
fn ret_res<T>(res: i32, ret: T) -> PassthruResult<T> {
    match res {
        0 => Ok(ret),
        _ => {
            log::error!("Function call failed with status {}", res);
            Err(PassthruError::try_from(res as u32).unwrap())
        },
    }
}

impl PassthruDrv {
    pub fn load_lib(path: String) -> Result<PassthruDrv, libloading::Error> {
        log::debug!("Opening function library {}", path);
        let lib = unsafe { Library::new(path)? };
        unsafe {
            let open_fn = *lib.get::<PassThruOpenFn>(b"PassThruOpen\0")?.into_raw();
            let close_fn = *lib.get::<PassThruCloseFn>(b"PassThruClose\0")?.into_raw();
            let connect_fn = *lib
                .get::<PassThruConnectFn>(b"PassThruConnect\0")?
                .into_raw();
            let disconnect_fn = *lib
                .get::<PassThruDisconnectFn>(b"PassThruDisconnect\0")?
                .into_raw();
            let read_msg_fn = *lib
                .get::<PassThruReadMsgsFn>(b"PassThruReadMsgs\0")?
                .into_raw();
            let write_msg_fn = *lib
                .get::<PassThruWriteMsgsFn>(b"PassThruWriteMsgs\0")?
                .into_raw();
            let start_periodic_fn = *lib
                .get::<PassThruStartPeriodicMsgFn>(b"PassThruStartPeriodicMsg\0")?
                .into_raw();
            let stop_periodic_fn = *lib
                .get::<PassThruStopPeriodicMsgFn>(b"PassThruStopPeriodicMsg\0")?
                .into_raw();
            let start_filter_fn = *lib
                .get::<PassThruStartMsgFilterFn>(b"PassThruStartMsgFilter\0")?
                .into_raw();
            let stop_filter_fn = *lib
                .get::<PassThruStopMsgFilterFn>(b"PassThruStopMsgFilter\0")?
                .into_raw();
            let set_prog_v_fn = *lib
                .get::<PassThruSetProgrammingVoltageFn>(b"PassThruSetProgrammingVoltage\0")?
                .into_raw();
            let get_last_err_fn = *lib
                .get::<PassThruGetLastErrorFn>(b"PassThruGetLastError\0")?
                .into_raw();
            let ioctl_fn = *lib.get::<PassThruIoctlFn>(b"PassThruIoctl\0")?.into_raw();
            let read_version_fn = *lib
                .get::<PassThruReadVersionFn>(b"PassThruReadVersion\0")?
                .into_raw();
            
            Ok(PassthruDrv {
                lib: Arc::new(lib),
                is_connected: false,
                open_fn,
                close_fn,
                connect_fn,
                disconnect_fn,
                read_msg_fn,
                write_msg_fn,
                start_periodic_fn,
                stop_periodic_fn,
                start_filter_fn,
                stop_filter_fn,
                set_prog_v_fn,
                get_last_err_fn,
                ioctl_fn,
                read_version_fn,
            })
        }
    }

    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    //type PassThruOpenFn = unsafe extern "stdcall" fn(name: *const libc::c_void, device_id: *mut u32) -> i32;
    pub fn open(&mut self) -> PassthruResult<u32> {
        log::debug!("PT_OPEN called");
        let mut id: u32 = 0;
        let res = unsafe { (&self.open_fn)(std::ptr::null(), &mut id) };
        if res == 0x00 {
            self.is_connected = true;
        }
        ret_res(res, id)
    }

    //type PassThruCloseFn = unsafe extern "stdcall" fn(device_id: u32) -> i32;
    pub fn close(&mut self, dev_id: u32) -> PassthruResult<()> {
        log::debug!("PT_CLOSE called. Device ID: {}", dev_id);
        let res = unsafe { (&self.close_fn)(dev_id) };
        if res == 0x00 {
            self.is_connected = false;
        }
        ret_res(res, ())
    }

    // type PassThruWriteMsgsFn = unsafe extern "stdcall" fn(channel_id: u32, msgs: *mut PASSTHRU_MSG, num_msgs: *mut u32, timeout: u32) -> i32;
    #[allow(trivial_casts)]
    pub fn write_messages(
        &self,
        channel_id: u32,
        msgs: &mut [PASSTHRU_MSG],
        timeout: u32,
    ) -> PassthruResult<usize> {
        log::debug!("PT_WRITE_MSGS called. Channel ID: {}, {} msgs, Timeout {}", channel_id, msgs.len(), timeout);
        if msgs.is_empty() {
            // No messages? Just tell application everything is OK
            return Ok(0);
        }
        let mut msg_count: u32 = msgs.len() as u32;
        let res = unsafe {
            (&self.write_msg_fn)(
                channel_id,
                msgs.as_mut_ptr(),
                &mut msg_count as *mut u32,
                timeout,
            )
        };
        ret_res(res, msg_count as usize)
    }

    //type PassThruReadMsgsFn = unsafe extern "stdcall" fn(channel_id: u32, msgs: *mut PASSTHRU_MSG, num_msgs: *mut u32, timeout: u32) -> i32;
    pub fn read_messages(
        &self,
        channel_id: u32,
        max_msgs: u32,
        timeout: u32,
    ) -> PassthruResult<Vec<PASSTHRU_MSG>> {
        //log::debug!("PT_READ_MSGS called. Channel ID: {}, {} msgs, Timeout {}", channel_id, max_msgs, timeout);
        let mut msg_count: u32 = max_msgs;
        // Create a blank array of empty passthru messages according to the max we should read
        let mut write_array: Vec<PASSTHRU_MSG> = vec![
            PASSTHRU_MSG {
                protocol_id: 0,
                rx_status: 0,
                tx_flags: 0,
                timestamp: 0,
                data_size: 0,
                extra_data_size: 0,
                data: [0; 4128]
            };
            max_msgs as usize
        ];

        let res = unsafe {
            (&self.read_msg_fn)(
                channel_id,
                write_array.as_mut_ptr(),
                &mut msg_count,
                timeout,
            )
        };
        if res == PassthruError::ERR_BUFFER_EMPTY as i32 {
            write_array.truncate(msg_count as usize);
            return ret_res(0x00, write_array);
        }
        if res == PassthruError::ERR_TIMEOUT as i32 {
            write_array.truncate(msg_count as usize);
            return ret_res(0x00, write_array);
        }
        if msg_count != max_msgs {
            // Trim the output vector to size
            write_array.truncate(msg_count as usize);
        }
        ret_res(res, write_array)
    }

    //type PassThruReadVersionFn = unsafe extern "stdcall" fn(device_id: u32, firmware_version: *mut libc::c_char, dll_version: *mut libc::c_char, api_version: *mut libc::c_char) -> i32;
    pub fn get_version(&self, dev_id: u32) -> PassthruResult<DrvVersion> {
        log::debug!("PT_GET_VERSION called. Device ID {}", dev_id);
        let mut firmware_version: [u8; 80] = [0; 80];
        let mut dll_version: [u8; 80] = [0; 80];
        let mut api_version: [u8; 80] = [0; 80];
        let res = unsafe {
            (&self.read_version_fn)(
                dev_id,
                firmware_version.as_mut_ptr() as *mut c_char,
                dll_version.as_mut_ptr() as *mut c_char,
                api_version.as_mut_ptr() as *mut c_char,
            )
        };
        unsafe {
            ret_res(
                res,
                DrvVersion {
                    api_version: CStr::from_ptr(api_version.as_ptr() as *const c_char)
                        .to_str()
                        .unwrap()
                        .to_string(),
                    dll_version: CStr::from_ptr(dll_version.as_ptr() as *const c_char)
                        .to_str()
                        .unwrap()
                        .to_string(),
                    fw_version: CStr::from_ptr(firmware_version.as_ptr() as *const c_char)
                        .to_str()
                        .unwrap()
                        .to_string(),
                },
            )
        }
    }

    //type PassThruGetLastErrorFn = unsafe extern "stdcall" fn(error_description: *mut libc::c_char) -> i32;
    pub fn get_last_error(&self) -> PassthruResult<String> {
        let mut err: [u8; 80] = [0; 80];
        let res = unsafe { (&self.get_last_err_fn)(err.as_mut_ptr() as *mut c_char) };
        ret_res(res, String::from_utf8(err.to_vec()).unwrap())
    }

    //type PassThruIoctlFn = unsafe extern "stdcall" fn(handle_id: u32, ioctl_id: u32, input: *mut libc::c_void, output: *mut libc::c_void) -> i32;
    pub fn ioctl(
        &self,
        handle_id: u32,
        ioctl_id: IoctlID,
        input: *mut c_void,
        output: *mut c_void,
    ) -> PassthruResult<()> {
        log::debug!("PT_IOCTL called. handle ID {}, IOCTL ID {}", handle_id, ioctl_id);
        let res = unsafe { (&self.ioctl_fn)(handle_id, ioctl_id as u32, input, output) };
        ret_res(res, ())
    }

    //type PassThruConnectFn = unsafe extern "stdcall" fn(device_id: u32, protocol_id: u32, flags: u32, baudrate: u32, channel_id: *mut u32) -> i32;
    /// Returns channel ID
    pub fn connect(
        &self,
        dev_id: u32,
        protocol: Protocol,
        flags: u32,
        baud: u32,
    ) -> PassthruResult<u32> {
        log::debug!("PT_CONNECT called. Device ID {}, protocol {}, flags: {:08X?}, baud: {}", dev_id, protocol, flags, baud);
        let mut channel_id: u32 = 0;
        let res =
            unsafe { (&self.connect_fn)(dev_id, protocol as u32, flags, baud, &mut channel_id) };
        ret_res(res, channel_id)
    }

    //type PassThruDisconnectFn = unsafe extern "stdcall" fn(channel_id: u32) -> i32;
    pub fn disconnect(&self, channel_id: u32) -> PassthruResult<()> {
        log::debug!("PT_DISCONNECT called. Channel ID {}", channel_id);
        ret_res(unsafe { (&self.disconnect_fn)(channel_id) }, ())
    }

    //type PassThruStartPeriodicMsgFn = unsafe extern "stdcall" fn(channel_id: u32, msg: *const PASSTHRU_MSG, msg_id: *mut u32, time_interval: u32) -> i32;
    /// Returns message ID
    #[allow(dead_code)]
    pub fn start_periodic_msg(
        &self,
        channel_id: u32,
        msg: &PASSTHRU_MSG,
        time_interval: u32,
    ) -> PassthruResult<u32> {
        let mut msg_id: u32 = 0;
        let res = unsafe { (&self.start_periodic_fn)(channel_id, msg, &mut msg_id, time_interval) };
        ret_res(res, msg_id)
    }

    //type PassThruStopPeriodicMsgFn = unsafe extern "stdcall" fn(channel_id: u32, msg_id: u32) -> i32;
    #[allow(dead_code)]
    pub fn stop_periodic_msg(&self, channel_id: u32, msg_id: u32) -> PassthruResult<()> {
        ret_res(unsafe { (&self.stop_periodic_fn)(channel_id, msg_id) }, ())
    }

    //type PassThruStartMsgFilterFn = unsafe extern "stdcall" fn(channel_id: u32, filter_type: u32, m_msg: *const PASSTHRU_MSG, p_msg: *const PASSTHRU_MSG, fc_msg: *const PASSTHRU_MSG, filter_id: *mut u32) -> i32;
    /// Returns filter ID
    pub fn start_msg_filter(
        &self,
        channel_id: u32,
        filter_type: FilterType,
        mask: &PASSTHRU_MSG,
        pattern: &PASSTHRU_MSG,
        flow_control: Option<PASSTHRU_MSG>,
    ) -> PassthruResult<u32> {
        log::debug!("PT_START_MSG_FILTER called. Channel ID {}", channel_id);
        let tmp = filter_type as u32;
        if tmp == FLOW_CONTROL_FILTER as u32 && flow_control.is_none() {
            return Err(PassthruError::ERR_INVALID_FILTER_ID);
        }

        let mut filter_id: u32 = 0;
        let res = match flow_control.as_ref() {
            None => unsafe {
                (&self.start_filter_fn)(
                    channel_id,
                    tmp,
                    mask,
                    pattern,
                    std::ptr::null(),
                    &mut filter_id,
                )
            },
            Some(fc) => unsafe {
                (&self.start_filter_fn)(channel_id, tmp, mask, pattern, fc, &mut filter_id)
            },
        };
        ret_res(res, filter_id)
    }

    //type PassThruStopMsgFilterFn = unsafe extern "stdcall" fn(channel_id: u32, filter_id: u32) -> i32;
    pub fn stop_msg_filter(&self, channel_id: u32, filter_id: u32) -> PassthruResult<()> {
        log::debug!("PT_STOP_MSG_FILTER called. Channel ID {}, Filter ID {}", channel_id, filter_id);
        let res = unsafe { (&self.stop_filter_fn)(channel_id, filter_id) };
        match res {
            0 => Ok(()),
            _ => Err(PassthruError::try_from(res as u32).unwrap()),
        }
    }

    //type PassThruSetProgrammingVoltageFn = unsafe extern "stdcall" fn(device_id: u32, pin_number: u32, voltage: u32) -> i32;
    #[allow(dead_code)]
    pub fn set_programming_voltage(
        &self,
        dev_id: u32,
        pin: u32,
        voltage: u32,
    ) -> PassthruResult<()> {
        ret_res(unsafe { (&self.set_prog_v_fn)(dev_id, pin, voltage) }, ())
    }
}
