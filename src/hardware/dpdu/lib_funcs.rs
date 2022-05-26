//! Module for interfacing with a PDU device's library

use std::{sync::Arc, ptr, borrow::BorrowMut, time::Duration};
use dpdu_rust::*;
use libloading::Library;

/// PDU API Result type
pub type PDUResult<T> = Result<T, PduError>;

#[derive(Clone)]
/// PDU API driver
pub struct PduDrv {
    /// Loaded library to interface with the device
    lib: Arc<Library>,
    /// Is the device currently connected?
    is_connected: bool,
    /// Construct function
    construct_fn: PduConstructFn,
    /// Desctruct function
    destruct_fn: PduDestructFn,
    /// IOCTL function
    ioctl_fn: PduIoctlFn,
    /// Get version function
    get_version_fn: PduGetVersionFn,
    /// Get status function
    get_status_fn: PduGetStatusFn,
    /// Get last error function
    get_last_error_fn: PduGetListErrorFn,
    /// Get resource status function
    get_resources_status_fn: PduGetResourceStatusFn,
    /// Create ComLogicalLink function
    create_cll_fn: PduCreateComLogicalLinkFn,
    /// Destroy ComLogicalLink function
    destroy_cll_fn: PduDestroyComLogicalLinkFn,
    /// Connect function
    connect_fn: PduConnectFn,
    /// Disconnect function
    disconnect_fn: PduDisconnectFn,
    /// Lock resource function
    lock_resource_fn: PduLockResourceFn,
    /// Unlock resource function
    unlock_resource_fn: PduUnlockResourceFn,
    /// Get ComParam function
    get_cp_fn: PduGetComParamFn,
    /// Set ComParam function
    set_cp_fn: PduSetComParamFn,
    /// Start ComPrimitive function]
    start_cp_fn: PduStartComPrimitiveFn,
    /// Cancel ComPrimitive function
    cancel_cp_fn: PduCancelComPrimitiveFn,
    /// Get event function
    get_evt_item_fn: PduGetEventItemFn,
    /// Destroy item function
    destroy_item_fn: PduDestroyItemFn,
    /// Register callback function
    register_callback_fn: PduRegisterCallbackFn,
    /// Get object ID function
    get_obj_id_fn: PduGetObjectIdFn,
    /// Get module ID function
    get_module_ids_fn: PduGetModuleIdsFn,
    /// Get resource ID function
    get_res_ids_fn: PduGetResourceIdsFn,
    /// Get conflicting resources function
    get_conflicting_res_fn: PduGetConflictingResourcesFn,
    /// Get unique resp ID table
    get_unique_resp_id_table_fn: PduGetUniqueRespIdTableFn,
    /// Set unique resp ID table
    set_unqiue_resp_id_table_fn: PduSetUniqueRespIdTableFn,
    /// Module connect function
    module_connect_fn: PduModuleConnectFn,
    /// Module disconnect function
    module_disconnect_fn: PduModuleDisconnectFn,
    /// get timestamp function
    get_timestamp_fn: PduGetTimestampFn
}


impl PduDrv {
    /// Attempts to load the library
    pub fn load_lib(path: String) -> Result<PduDrv, libloading::Error> {
        log::debug!("Opening function library {}", path);
        let lib = unsafe { Library::new(path)? };
        unsafe {
            let construct_fn = *lib.get::<PduConstructFn>(b"PDUConstruct\0")?.into_raw();
            let destruct_fn = *lib.get::<PduDestructFn>(b"PDUDestruct\0")?.into_raw();
            let ioctl_fn = *lib.get::<PduIoctlFn>(b"PDUIoCtl\0")?.into_raw();
            let get_version_fn = *lib.get::<PduGetVersionFn>(b"PDUGetVersion\0")?.into_raw();
            let get_status_fn = *lib.get::<PduGetStatusFn>(b"PDUGetStatus\0")?.into_raw();
            let get_last_error_fn = *lib.get::<PduGetListErrorFn>(b"PDUGetLastError\0")?.into_raw();
            let get_resources_status_fn = *lib.get::<PduGetResourceStatusFn>(b"PDUGetResourceStatus\0")?.into_raw();
            let create_cll_fn = *lib.get::<PduCreateComLogicalLinkFn>(b"PDUCreateComLogicalLink\0")?.into_raw();
            let destroy_cll_fn = *lib.get::<PduDestroyComLogicalLinkFn>(b"PDUDestroyComLogicalLink\0")?.into_raw();
            let connect_fn = *lib.get::<PduConnectFn>(b"PDUConnect\0")?.into_raw();
            let disconnect_fn = *lib.get::<PduDisconnectFn>(b"PDUDisconnect\0")?.into_raw();
            let lock_resource_fn = *lib.get::<PduLockResourceFn>(b"PDULockResource\0")?.into_raw();
            let unlock_resource_fn = *lib.get::<PduUnlockResourceFn>(b"PDUUnlockResource\0")?.into_raw();
            let get_cp_fn = *lib.get::<PduGetComParamFn>(b"PDUGetComParam\0")?.into_raw();
            let set_cp_fn = *lib.get::<PduSetComParamFn>(b"PDUSetComParam\0")?.into_raw();
            let start_cp_fn = *lib.get::<PduStartComPrimitiveFn>(b"PDUStartComPrimitive\0")?.into_raw();
            let cancel_cp_fn = *lib.get::<PduCancelComPrimitiveFn>(b"PDUCancelComPrimitive\0")?.into_raw();
            let get_evt_item_fn = *lib.get::<PduGetEventItemFn>(b"PDUGetEventItem\0")?.into_raw();
            let destroy_item_fn = *lib.get::<PduDestroyItemFn>(b"PDUDestroyItem\0")?.into_raw();
            let register_callback_fn = *lib.get::<PduRegisterCallbackFn>(b"PDURegisterEventCallback\0")?.into_raw();
            let get_obj_id_fn = *lib.get::<PduGetObjectIdFn>(b"PDUGetObjectId\0")?.into_raw();
            let get_module_ids_fn = *lib.get::<PduGetModuleIdsFn>(b"PDUGetModuleIds\0")?.into_raw();
            let get_res_ids_fn = *lib.get::<PduGetResourceIdsFn>(b"PDUGetResourceIds\0")?.into_raw();
            let get_conflicting_res_fn = *lib.get::<PduGetConflictingResourcesFn>(b"PDUGetConflictingResources\0")?.into_raw();
            let get_unique_resp_id_table_fn = *lib.get::<PduGetUniqueRespIdTableFn>(b"PDUGetUniqueRespIdTable\0")?.into_raw();
            let set_unqiue_resp_id_table_fn = *lib.get::<PduSetUniqueRespIdTableFn>(b"PDUSetUniqueRespIdTable\0")?.into_raw();
            let module_connect_fn = *lib.get::<PduModuleConnectFn>(b"PDUModuleConnect\0")?.into_raw();
            let module_disconnect_fn = *lib.get::<PduModuleDisconnectFn>(b"PDUModuleDisconnect\0")?.into_raw();
            let get_timestamp_fn = *lib.get::<PduGetTimestampFn>(b"PDUGetTimestamp\0")?.into_raw();
            
            Ok(Self {
                lib:Arc::new(lib),
                is_connected: false,
                construct_fn,
                destruct_fn,
                ioctl_fn,
                get_version_fn,
                get_status_fn,
                get_last_error_fn,
                get_resources_status_fn,
                create_cll_fn,
                destroy_cll_fn,
                connect_fn,
                disconnect_fn,
                lock_resource_fn,
                unlock_resource_fn,
                get_cp_fn,
                set_cp_fn,
                start_cp_fn,
                cancel_cp_fn,
                get_evt_item_fn,
                destroy_item_fn,
                register_callback_fn,
                get_obj_id_fn,
                get_module_ids_fn,
                get_res_ids_fn,
                get_conflicting_res_fn,
                get_unique_resp_id_table_fn,
                set_unqiue_resp_id_table_fn,
                module_connect_fn,
                module_disconnect_fn,
                get_timestamp_fn,
            })
        }
    }
}


impl std::fmt::Debug for PduDrv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PduDrv").finish()
    }
}

impl PduDrv {
    /// Constructs the PDU API
    /// This does NOT support API Tag
    pub fn construct(&mut self, option_str: String) -> PDUResult<()> {
        let mut c = option_str;
        match (&self.construct_fn)(c.as_mut_ptr(), ptr::null_mut()) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Destructs the PDU API
    /// This does NOT support API Tag
    pub fn destruct(&mut self) -> PDUResult<()> {
        match (&self.destruct_fn)() {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Gets version information
    pub fn get_version(&mut self, vci_handle: u32) -> PDUResult<VersionData> {
        let mut x: VersionData = unsafe { std::mem::zeroed() };
        match (&self.get_version_fn)(vci_handle, &mut x) {
            PduError::StatusNoError => Ok(x),
            e => Err(e)
        }
    }

    /// Gets status
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle
    /// * com_logical_link_handle - Raw ComLogicalLink handle
    /// * com_primitive - Handle for ComPrimitive to request
    /// 
    /// ## Notes
    /// * vci_handle and com_logical_link_handle can not be BOTH None
    /// 
    /// ## Returns
    /// This function will return either a success or error depending if the function succeeded. 
    /// In the event of success, the following 3 things will be returned in a tuple
    /// 1. The status
    /// 2. The timestamp of the status event (Microseconds)
    /// 3. Optional extra information
    pub fn get_status(&mut self, vci_handle: Option<u32>, com_logical_link_handle: Option<u32>, com_primitive: u32) -> PDUResult<(PduStatus, u32, Option<ExtraInfo>)> {
        if vci_handle.is_none() && com_logical_link_handle.is_none() {
            return Err(PduError::InvalidParameters)
        }
        let vci = vci_handle.unwrap_or(PDU_HANDLE_UNDEF);
        let cll = vci_handle.unwrap_or(PDU_HANDLE_UNDEF);

        let mut status : PduStatus = unsafe { std::mem::zeroed() };
        let mut timestamp: u32 = 0;
        let mut extra_info_ptr = 0;

        match (&self.get_status_fn)(vci, cll, com_primitive, &mut status, &mut timestamp, &mut extra_info_ptr) {
            PduError::StatusNoError => {
                let extra_info = match extra_info_ptr {
                    0 => None,
                    _ => unsafe { Some(*Box::from_raw(extra_info_ptr as *mut ExtraInfo)) }
                };
                Ok((
                    status,
                    timestamp,
                    extra_info
                ))
            },
            e => Err(e)
        }
    }

    /// Returns the last error from the API. This is only applicable to J2534-2 adapters
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI Handle
    /// * com_logical_link_handle - Raw ComLogicalLink handle
    /// * com_primitive_handle - Optional handle for ComPrimitive should the error being requested attain to one
    /// 
    /// ## Notes
    /// This function call might be successful even if there is NO last error to request,
    /// in which case the first value of the tuple will be [PduErrorEvt::NoError]
    /// 
    /// ## Returns
    /// This function will return either a success or error depending if the function succeeded. 
    /// In the event of success, the following 3 things will be returned in a tuple
    /// 1. The error event type
    /// 2. The timestamp of the error event (Microseconds)
    /// 3. Optional extra information about the error event
    pub fn get_last_error(&mut self, vci_handle: u32, com_logical_link_handle: u32, com_primitive_handle: Option<u32>) -> PDUResult<(PduErrorEvt, u32, Option<ExtraInfo>)> {
        let mut cph = com_primitive_handle.unwrap_or(PDU_HANDLE_UNDEF);

        let mut event : PduErrorEvt = unsafe { std::mem::zeroed() };
        let mut timestamp: u32 = 0;
        let mut extra_info_ptr = 0;

        match (&self.get_last_error_fn)(vci_handle, com_logical_link_handle, &mut event, &mut cph, &mut timestamp, &mut extra_info_ptr) {
            PduError::StatusNoError => {
                let extra_info = match extra_info_ptr {
                    0 => None,
                    _ => unsafe { Some(*Box::from_raw(extra_info_ptr as *mut ExtraInfo)) }
                };
                Ok((
                    event,
                    timestamp,
                    extra_info
                ))
            },
            e => Err(e)
        }
    }

    /// Gets the resource status of the requested resource of a VCI
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to request the status from
    /// * resource_id - ID of the resource to request
    pub fn get_resource_status(&mut self, vci_handle: u32, resource_id: u32) -> PDUResult<u32> {
        let mut resource_status = RscStatusItem {
            h_mod: vci_handle,
            resource_id,
            resource_status: 0,
        };
        match (&self.get_resources_status_fn)(&mut resource_status) {
            PduError::StatusNoError => Ok(resource_status.resource_status),
            e => Err(e)
        }
    }

    /// Creates a COM Logical link on the VCI with the vehicle
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI Handle to create a COM logical link on
    /// * resource_data - Pointer to resource data for the logical link (To be filled out before calling this function)
    /// * resource_id - Resource ID for settings of the ComLogicalLink
    /// 
    /// ## Notes
    /// * This function does NOT yet support setting the API Tag
    /// 
    /// ## Returns
    /// This function will return either a success or error depending if the function succeeded. 
    /// In the event of success, the following 2 things will be returned in a tuple
    /// 
    /// 1. The ID of the created ComLogicalLink handle
    //  2. The flag of the created ComLogicalLink
    pub fn create_com_logical_link(&mut self, vci_handle: u32, resource_data: &mut RscData, resource_id: u32) -> PDUResult<(u32, FlagData)> {
        let mut cll_id: u32 = 0;
        let mut cll_flag: FlagData = unsafe { std::mem::zeroed() };
        match (&self.create_cll_fn)(vci_handle, resource_data, resource_id, ptr::null_mut(), &mut cll_id, &mut cll_flag) {
            PduError::StatusNoError => Ok((cll_id, cll_flag)),
            e => Err(e)
        }
    }

    /// Destroys a created ComLogicalLink
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to VCI to destroy the Com logical link
    /// * logical_link_handle - Raw Handle of the logical link to destroy. Created by [create_com_logical_link]
    pub fn destroy_com_logical_link(&mut self, vci_handle: u32, logical_link_handle: u32) -> PDUResult<()> {
        match (&self.destroy_cll_fn)(vci_handle, logical_link_handle) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Tries to connect a created ComLogicalLink
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to VCI to connect the Com logical link
    /// * logical_link_handle - Raw Handle of the logical link to connect. Created by [create_com_logical_link]
    pub fn connect(&mut self, vci_handle: u32, logical_link_handle: u32) -> PDUResult<()> {
        match (&self.connect_fn)(vci_handle, logical_link_handle) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Tries to disconnect a created ComLogicalLink
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to VCI to disconnect the Com logical link
    /// * logical_link_handle - Raw Handle of the logical link to disconnect. Created by [create_com_logical_link]
    pub fn disconnect(&mut self, vci_handle: u32, logical_link_handle: u32) -> PDUResult<()> {
        match (&self.disconnect_fn)(vci_handle, logical_link_handle) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Tries to lock a resource, which allows a ComLogicalLink exclusive access
    /// to one of a vehicle's communication interface
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to VCI
    /// * logical_link_handle - Raw Handle of the logical link to lock
    /// * lock_mask - Bit encoded mask which encodes the lock resource request
    pub fn lock_resource(&mut self, vci_handle: u32, logical_link_handle: u32, lock_mask: u32) -> PDUResult<()> {
        match (&self.lock_resource_fn)(vci_handle, logical_link_handle, lock_mask) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Tries to unlock a resource that was previously locked with [lock_resource],
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to VCI
    /// * logical_link_handle - Raw Handle of the logical link to unlock
    /// * lock_mask - Bit encoded mask which encodes the unlock resource request
    pub fn unlock_resource(&mut self, vci_handle: u32, logical_link_handle: u32, lock_mask: u32) -> PDUResult<()> {
        match (&self.unlock_resource_fn)(vci_handle, logical_link_handle, lock_mask) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Gets a ComParam
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to VCI
    /// * logical_link_handle - Raw Handle of the logical link to request the ComParam from
    /// * param_id - ID of the ComParam to be requested
    pub fn get_com_param(&mut self, vci_handle: u32, logical_link_handle: u32, param_id: u32) -> PDUResult<ParamItem> {
        let mut param_item: ParamItem = unsafe { std::mem::zeroed() };
        let mut param_item_ptr: *mut ParamItem = &mut param_item;
        match (&self.get_cp_fn)(vci_handle, logical_link_handle, param_id, &mut param_item_ptr) {
            PduError::StatusNoError => Ok(param_item),
            e => Err(e)
        }
    }

    /// Sets a ComParam
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to VCI
    /// * logical_link_handle - Raw Handle of the logical link to request the ComParam from
    /// * param_item - ComParam to set
    pub fn set_com_param(&mut self, vci_handle: u32, logical_link_handle: u32, param_item: &mut ParamItem) -> PDUResult<()> {
        match (&self.set_cp_fn)(vci_handle, logical_link_handle, param_item) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Starts a Com Primitive on a ComLogicalLink
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to VCI
    /// * logical_link_handle - Raw Handle of the logical link to start the ComPrimitive on
    /// * cp_type - The type of ComPrimitive to start
    /// * data - Data for the ComPrimitive (Can be empty)
    /// * ctrl - Control data for the ComPrimitive (Can be null)
    /// 
    /// ## Notes
    /// * This function does not yet support setting the pCopTag
    /// 
    /// ## Returns
    /// This function if successful will return the raw handle of the ComPrimitive that was started
    pub fn start_com_primitive(&mut self, vci_handle: u32, logical_link_handle: u32, cp_type: PduCopt, data: &mut [u8], ctrl: &mut CopCtrlData) -> PDUResult<u32> {

        let data_ptr: *mut u8 = if data.len() == 0 { ptr::null_mut() } else { data.as_mut_ptr() };

        let data_size = data.len() as u32;
        let mut handle = 0;
        match (&self.start_cp_fn)(vci_handle, logical_link_handle, cp_type, data_size, data_ptr, ctrl, ptr::null_mut(), &mut handle) {
            PduError::StatusNoError => Ok(handle),
            e => Err(e)
        }
    }

    /// Stops a created a Com Primitive on a ComLogicalLink
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle to VCI
    /// * logical_link_handle - Raw Handle of the logical link to start the ComPrimitive on
    /// * com_primitive_handle - Raw handle of the Started ComPrimitive, see [start_com_primitive]
    pub fn cancel_com_primitive(&mut self, vci_handle: u32, logical_link_handle: u32, com_primitive_handle: u32) -> PDUResult<()> {
        match (&self.cancel_cp_fn)(vci_handle, logical_link_handle, com_primitive_handle) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Gets event item data for an event source
    /// 
    /// ## Parameters
    /// * vci_handle - Optional VCI handle
    /// * logical_link_handle - optional ComLogicalLink handle
    /// 
    /// ## Notes
    /// * In the event that `vci_handle` is `None`, `logical_link_handle` cannot be `Some`
    pub fn get_event_item(&mut self, vci_handle: Option<u32>, logical_link_handle: Option<u32>, event_data_prefilled: EventItem) -> PDUResult<Option<EventItem>> {
        if vci_handle.is_none() && logical_link_handle.is_some() { // Invalid input state
            return Err(PduError::InvalidParameters)
        }
        let vci = vci_handle.unwrap_or(PDU_HANDLE_UNDEF);
        let cll = vci_handle.unwrap_or(PDU_HANDLE_UNDEF);

        let mut event = event_data_prefilled;
        let mut ptr: *mut EventItem = event.borrow_mut();

        match (&self.get_evt_item_fn)(vci, cll, &mut ptr) {
            PduError::StatusNoError => {
                if ptr.is_null() {
                    Ok(None)
                } else {
                    Ok(Some(event))
                }
            },
            e => Err(e)
        }

    }

    /// Destroys an item created earlier
    /// 
    /// ## Parameters
    /// * item - Pointer to item to be destroyed
    pub fn destroy_item(&mut self, item: *mut PduItem) -> PDUResult<()> {
        match (&self.destroy_item_fn)(item) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Registers a callback for the PDU API
    /// 
    /// ## Parameters
    /// * vci_handle - Raw VCI handle, if set to none then callback will be used for system callbacks
    /// * com_logical_link_handle - Raw Handle of the ComLogicalLink. If None, then callback will be for system or module callbacks
    /// * callback - The callback function
    pub fn register_callback(&mut self, vci_handle: Option<u32>, com_logical_link_handle: Option<u32>, callback: EventCallbackFn) -> PDUResult<()> {
        let vci = vci_handle.unwrap_or(PDU_HANDLE_UNDEF);
        let cll = com_logical_link_handle.unwrap_or(PDU_HANDLE_UNDEF);
        match (&self.register_callback_fn)(vci, cll, callback) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }


    /// Gets an object ID given an object name from the VCI
    /// 
    /// ## Parameters
    /// * object_type - The type of object to request
    /// * name - The short name of the object to request
    /// 
    /// ## Returns
    /// If this function is successful, then an optional ID of the requested object will be returned.
    /// This function can also return `Ok(None)` which would imply the function call was successful,
    /// but no matching object ID was found.
    pub fn get_object_id<T: Into<String>>(&mut self, object_type: PduObjt, name: T) -> PDUResult<Option<u32>> {
        let mut c: String = name.into();
        let mut dest: u32 = PDU_ID_UNDEF;
        match (&self.get_obj_id_fn)(object_type, c.as_mut_ptr(), &mut dest) {
            PduError::StatusNoError => {
                if dest == PDU_ID_UNDEF {
                    Ok(None)
                } else {
                    Ok(Some(dest))
                }
            },
            e => Err(e)
        }
    }

    /// Gets a list of module IDs and their status
    pub fn get_module_ids(&mut self) -> PDUResult<ModuleItem> {
        let l: *mut *mut ModuleItem = std::ptr::null_mut();
        match (&self.get_module_ids_fn)(l) {
            PduError::StatusNoError => {
                // Take pointer of pointer and read it to returned data
                Ok(unsafe { Box::from_raw(l).read() })
            }
            e => Err(e)
        }
    }

    /// Gets resource IDs from the PDU API
    /// 
    /// ## Parameters
    /// * vci_handle - Optional VCI handle to get resource IDs from. If None, then 
    ///                 it will return ALL the resource IDs, regardless of the device
    /// 
    /// ## Returns
    /// If this function is successful, then 2 things are returned
    /// 1. The resource ID Data
    /// 2. The resource ID list
    pub fn get_resource_ids(&mut self, vci_handle: Option<u32>) -> PDUResult<(RscData, RscIdItem)> {
        let vci = vci_handle.unwrap_or(PDU_HANDLE_UNDEF);
        let list: *mut *mut RscIdItem = std::ptr::null_mut();
        let mut data: RscData = unsafe { std::mem::zeroed() };
        match (&self.get_res_ids_fn)(vci, &mut data, list) {
            PduError::StatusNoError => {
                Ok((
                    data,
                    unsafe { Box::from_raw(list).read() }
                ))
            }
            e => Err(e)
        }
    }

    /// Gets conflicting resources from PDU API
    /// 
    /// ## Parameters
    /// * resource_id - Resource ID to check
    /// * module_list - The list of modules to check [resource_id] against for conflicts
    pub fn get_conflicting_resources(&mut self, resource_id: u32, module_list: ModuleItem) -> PDUResult<RscConflictItem> {
        let list: *mut *mut RscConflictItem = std::ptr::null_mut();
        let mut m_list = module_list;
        match (&self.get_conflicting_res_fn)(resource_id, &mut m_list, list) {
            PduError::StatusNoError => {
                Ok(unsafe { Box::from_raw(list).read() })
            }
            e => Err(e)
        }
    }


    /// Gets unique response ID table
    /// 
    /// ## Parameters
    /// * vci_handle - Raw handle of the VCI
    /// * com_logical_link_handle - Raw handle of the ComLogicalLink
    pub fn get_unqiue_response_id_table(&mut self, vci_handle: u32, com_logical_link_handle: u32) -> PDUResult<UniqueRespIdTableItem> {
        let table: *mut *mut UniqueRespIdTableItem = std::ptr::null_mut();
        match (&self.get_unique_resp_id_table_fn)(vci_handle, com_logical_link_handle, table) {
            PduError::StatusNoError => {
                Ok(unsafe { Box::from_raw(table).read() })
            }
            e => Err(e)
        }
    }

    /// Sets the unique response ID table
    /// 
    /// ## Parameters
    /// * vci_handle - Raw handle of the VCI
    /// * com_logical_link_handle - Raw handle of the ComLogicalLink
    /// * unique_resp_table - Unique response table to set
    pub fn set_unqiue_response_id_table(&mut self, vci_handle: u32, com_logical_link_handle: u32, unique_resp_table: &mut UniqueRespIdTableItem) -> PDUResult<()> {
        match (&self.set_unqiue_resp_id_table_fn)(vci_handle, com_logical_link_handle, unique_resp_table) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Connects a VCI
    ///
    /// ## Parameters
    /// * vci_handle - Raw handle of the VCI to connect
    pub fn module_connect(&mut self, vci_handle: u32) -> PDUResult<()> {
        match (&self.module_connect_fn)(vci_handle) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Disconnects a VCI
    ///
    /// ## Parameters
    /// * vci_handle - Raw handle of the VCI to disconnect
    pub fn module_disconnect(&mut self, vci_handle: u32) -> PDUResult<()> {
        match (&self.module_disconnect_fn)(vci_handle) {
            PduError::StatusNoError => Ok(()),
            e => Err(e)
        }
    }

    /// Gets the timestamp from the VCI
    /// 
    /// ## Parameters
    /// * vci_handle - Raw handle of the VCI to get the timestamp from
    pub fn get_timestamp(&mut self, vci_handle: u32) -> PDUResult<Duration> {
        let mut p: u32 = 0;
        match (&self.get_timestamp_fn)(vci_handle, &mut p) {
            PduError::StatusNoError => Ok(Duration::from_micros(p as u64)),
            e => Err(e)
        }
    }
}   