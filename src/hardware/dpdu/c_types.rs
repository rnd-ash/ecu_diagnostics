//! C-Type definitions and helper functions for D-PDU API

use std::ffi;


#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
/// D-PDU API item types
pub enum T_PDU_IT {
    /// IOCTL unsigned 32 bit int
    PDU_IT_IO_UNUM32 = 0x1000,
    /// IOCTL Program voltage
    PDU_IT_IO_PROG_VOLTAGE = 0x1001,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
/// ComPrimitive type values
pub enum T_PDU_COPT {
    /// Start communication with ECU by sending an optional request
    PDU_COPT_STARTCOMM = 0x8001,
    /// Stop communication with ECU by sending an optional request
    PDU_COPT_STOPCOMM = 0x8002,
    /// Copies ComParams (see [E_PDU_IT]) to a ComLogicalLink from the working buffer to receive buffer
    PDU_COPT_UPDATEPARAM = 0x8003,
    /// Send request data and/or receive corresponding response data
    PDU_COPT_SENDRECV = 0x8004,
    /// Wait the given time span before executing the next ComPrimitive
    PDU_COPT_DELAY = 0x8005,
    /// Copies comParams related to a comLogicalLink from active buffer to working buffer
    PDU_COPT_RESTORE_PARAM = 0x8006,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
/// IOCTL filter type values
pub enum T_PDU_FILTER {
    /// Allows matching messages into the receive event queue, for all protocols
    PDU_FLT_PASS = 0x00000001,
    /// Keeps matching mressages out of the receive event queue, for all protocols
    PDU_FLT_BLOCK = 0x00000002,
    /// Allows matching messages into the receive event queue which are of UUDT type only (For ISO15765)
    PDU_FLT_PASS_UUDT = 0x00000011,
    /// Allows matching messages out of the receive event queue which are of UUDT type only (For ISO15765)
    PDU_FLT_PBLOCK_UUDT = 0x00000012
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
/// IOCTL event queue mode type
pub enum T_PDU_QUEUE_MODE {
    /// Unlimited receive buffer size (Keeps growing if full)
    PDU_QUE_UNLIMITED = 0x00000000,
    /// Limited receive buffer size. If full, new messages are dropped
    PDU_QUE_LIMITED = 0x00000001,
    /// Limited circular buffer. If full, new messages overwrite old messages - oldest first
    PDU_QUE_CIRCULAR = 0x00000002
}  


#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
/// PDU Error type
pub enum T_PDU_ERROR {
    PDU_STATUS_NOERROR              = 0x00000000,   /* No error for the function call */
    PDU_ERR_FCT_FAILED              = 0x00000001,   /* Function call failed (generic failure) */
    PDU_ERR_RESERVED_1              = 0x00000010,   /* Reserved by ISO22900-2 */
    PDU_ERR_COMM_PC_TO_VCI_FAILED   = 0x00000011,   /* Communication between host and MVCI Protocol Module failed */
    PDU_ERR_PDUAPI_NOT_CONSTRUCTED  = 0x00000020,   /* The D-PDU API has not yet been constructed */
    PDU_ERR_SHARING_VIOLATION       = 0x00000021,   /* A PDUDestruct was not called before another PDUConstruct */
    PDU_ERR_RESOURCE_BUSY           = 0x00000030,   /* The requested resource is already in use */
    PDU_ERR_RESOURCE_TABLE_CHANGED  = 0x00000031,   /* Not used by the D-PDU API */
    PDU_ERR_RESOURCE_ERROR          = 0x00000032,   /* Not used by the D-PDU API */
    PDU_ERR_CLL_NOT_CONNECTED       = 0x00000040,   /* The ComLogicalLink cannot be in the PDU_CLLST_OFFLINE state to 
                                                       perform the requested operation */
    PDU_ERR_CLL_NOT_STARTED         = 0x00000041,   /* The ComLogicalLink must be in the PDU_CLLST_COMM_STARTED state 
                                                       to perform the requested operation */
    PDU_ERR_INVALID_PARAMETERS      = 0x00000050,   /* One or more of the parameters supplied in the function are 
                                                       invalid */
    PDU_ERR_INVALID_HANDLE          = 0x00000060,   /* One or more of the handles supplied in the function are invalid */
    PDU_ERR_VALUE_NOT_SUPPORTED     = 0x00000061,   /* One of the option values in PDUConstruct is invalid */
    PDU_ERR_ID_NOT_SUPPORTED        = 0x00000062,   /* IOCTL command id not supported by the implementation of the 
                                                       D-PDU API */
    PDU_ERR_COMPARAM_NOT_SUPPORTED  = 0x00000063,   /* ComParam id not supported by the implementation of the D-PDU API */
    PDU_ERR_COMPARAM_LOCKED         = 0x00000064,   /* Physical ComParam cannot be changed because it is locked by 
                                                       another ComLogicalLink */
    PDU_ERR_TX_QUEUE_FULL           = 0x00000070,   /* The ComLogicalLinks transmit queue is full; the ComPrimitive 
                                                       could not be queued */
    PDU_ERR_EVENT_QUEUE_EMPTY       = 0x00000071,   /* No more event items are available to be read from the requested 
                                                       queue */
    PDU_ERR_VOLTAGE_NOT_SUPPORTED   = 0x00000080,   /* The voltage value supplied in the IOCTL call is not supported by 
                                                       the MVCI Protocol Module */
    PDU_ERR_MUX_RSC_NOT_SUPPORTED   = 0x00000081,   /* The specified pin / resource are not supported by the MVCI Protocol 
                                                       Module for the IOCTL call */
    PDU_ERR_CABLE_UNKNOWN           = 0x00000082,   /* The cable attached to the MVCI Protocol Module is of an unknown 
                                                       type */
    PDU_ERR_NO_CABLE_DETECTED       = 0x00000083,   /* No cable is detected by the MVCI Protocol Module */
    PDU_ERR_CLL_CONNECTED           = 0x00000084,   /* The ComLogicalLink is already in the PDU_CLLST_ONLINE state */
    PDU_ERR_TEMPPARAM_NOT_ALLOWED   = 0x00000090,   /* Physical ComParams cannot be changed as a temporary ComParam */
    PDU_ERR_RSC_LOCKED              = 0x000000A0,   /* The resource is already locked */
    PDU_ERR_RSC_LOCKED_BY_OTHER_CLL = 0x000000A1,   /* The ComLogicalLink's resource is currently locked by another 
                                                       ComLogicalLink */
    PDU_ERR_RSC_NOT_LOCKED          = 0x000000A2,   /* The resource is already in the unlocked state */
    PDU_ERR_MODULE_NOT_CONNECTED    = 0x000000A3,   /* The module is not in the PDU_MODST_READY state */
    PDU_ERR_API_SW_OUT_OF_DATE      = 0x000000A4,   /* The API software is older than the MVCI Protocol Module Software */
    PDU_ERR_MODULE_FW_OUT_OF_DATE   = 0x000000A5,   /* The MVCI Protocol Module software is older than the API software */
    PDU_ERR_PIN_NOT_CONNECTED       = 0x000000A6,   /* The requested Pin is not routed by supported cable */
    PDU_ERR_IP_PROTOCOL_NOT_SUPPORTED                     = 0x000000B0,  /* IP protocol is not supported: e.g. IPv6 used as protocolVersion, 
                                                                            but OS doesn't support IPv6 (or it is disabled).*/
    PDU_ERR_DOIP_ROUTING_ACTIVATION_FAILED                = 0x000000B1,  /* DoIP Routing activation failed */
    PDU_ERR_DOIP_ROUTING_ACTIVATION_AUTHENTICATION_FAILED = 0x000000B2,  /* DoIP Routing activation denied due to missing authentication */
    PDU_ERR_DOIP_AMBIGUOUS_LOGICAL_ADDRESS                = 0x000000B3   /* Denied to connect a DoIP LogicalLink with a logical address 
                                                                                   which is identical for multiple DoIP entities inside a DoIP MVCI module 
                                                                                  representing a collection of DoIP entities */
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types, non_snake_case)]
struct PDU_VERSION_DATA {
    MVCI_Part1StandardVersion: u32,
    MVCI_Part2StandardVersion: u32,
    HwSerialNumber: u32,
    HwName: [u8; 64],
    HwVersion: u32,
    HwDate: u32,
    HwInterface: u32,
    FwName: [u8; 64],
    FwVersion: u32,
    FwDate: u32,
    VendorName: [u8; 64],
    PDUApiSwName: [u8; 64],
    PDUApiSwVersion: u32,
    PDUApiSwDate: u32
}

impl Default for PDU_VERSION_DATA {
    fn default() -> Self {
        Self {
            MVCI_Part1StandardVersion: 0,
            MVCI_Part2StandardVersion: 0,
            HwSerialNumber: 0,
            HwName: [0; 64],
            HwVersion: 0,
            HwDate: 0,
            HwInterface: 0,
            FwName: [0; 64],
            FwVersion: 0,
            FwDate: 0,
            VendorName: [0; 64],
            PDUApiSwName: [0; 64],
            PDUApiSwVersion: 0,
            PDUApiSwDate: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
/// Holds a generic IOCTL parameter as a pointer with its type
pub struct PDU_DATA_ITEM {
    /// IOCTL type from [T_PDU_IT]
    ItemType: T_PDU_IT,
    /// Pointer to data that holds specific IOCTL data structure
    pData: *mut ffi::c_void
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
/// IOCTL programming voltage structure
pub struct PDU_IO_PROG_VOLTAGE_DATA {
    /// Programming voltage in mV
    prog_voltage_mv: u32,
    /// Pin number on data link connnector
    pin_on_dlc: u32
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
/// IOCTL byte array structure
pub struct PDU_IO_BYTEARRAY_DATA {
    data_size: u32,
    pData: *mut u8
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
/// IOCTL filter data struct
pub struct PDU_IO_FILTER_DATA {

}