//! Diagnostic implementation for the PCAN-USB API
//!
//! NOTE: This is only for Windows. For Linux, you can use PCAN-USB with the native SocketCAN driver

mod lib_funcs;
pub(crate) mod pcan_types;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

use crate::{
    channel::{CanChannel, CanFrame, ChannelError, ChannelResult, IsoTPChannel, PacketChannel},
    hardware::{
        pcan_usb::pcan_types::{PCANError, ALL_USB_DEVICES},
        HardwareCapabilities,
    },
};

use self::{
    lib_funcs::PCanDrv,
    pcan_types::{PCANBaud, PcanUSB},
};

use super::{
    software_isotp::SoftwareIsoTpChannel, Hardware, HardwareError, HardwareInfo, HardwareResult,
    HardwareScanner, IsoTpChannelType,
};

#[derive(Clone, Debug)]
/// PCAN USB device
pub struct PcanUsbDevice {
    info: HardwareInfo,
    dev_handle: PcanUSB,
    driver: PCanDrv,
    can_channel: Arc<AtomicBool>,
}

impl PcanUsbDevice {
    pub(crate) fn new(
        handle: PcanUSB,
        info: HardwareInfo,
        driver: PCanDrv,
    ) -> HardwareResult<Self> {
        // Destroy any handle if it exists (Device reset)
        let res = driver.reset_handle(handle);
        if let Err(HardwareError::APIError { code, desc: _ }) = res {
            if code != PCANError::Initialize as u32 {
                return Err(res.err().unwrap()); // Some other error. Non intialized error is OK since it means first use
            }
        }

        Ok(Self {
            info,
            dev_handle: handle,
            driver,
            can_channel: Arc::new(AtomicBool::new(false)),
        })
    }
}

impl Drop for PcanUsbDevice {
    fn drop(&mut self) {
        let _ = self.driver.reset_handle(self.dev_handle);
    }
}

impl Hardware for PcanUsbDevice {
    fn create_iso_tp_channel(
        &mut self,
        force_native: bool,
    ) -> HardwareResult<Box<dyn IsoTPChannel>> {
        if force_native {
            Err(HardwareError::ChannelNotSupported)
        } else {
            // Use software
            let can_channel = self.create_can_channel()?;
            let sw = SoftwareIsoTpChannel::new(can_channel);
            Ok(Box::new(sw))
        }
    }

    fn create_can_channel(&mut self) -> HardwareResult<Box<dyn CanChannel>> {
        if self.can_channel.load(Ordering::Relaxed) {
            // Already open
            Err(HardwareError::ConflictingChannel)
        } else {
            self.can_channel.store(true, Ordering::Relaxed);
            let s = PcanUsbpacketChannel {
                baud: None,
                use_ext: None,
                filter_active: false,
                dev_handle: self.dev_handle,
                open: false,
                driver: self.driver.clone(),
                device_state: self.can_channel.clone(),
            };
            Ok(Box::new(s))
        }
    }

    fn is_iso_tp_channel_open(&self) -> bool {
        false
    }

    fn is_can_channel_open(&self) -> bool {
        self.can_channel.load(Ordering::Relaxed)
    }

    fn read_battery_voltage(&mut self) -> Option<f32> {
        None
    }

    fn read_ignition_voltage(&mut self) -> Option<f32> {
        None
    }

    fn get_info(&self) -> &HardwareInfo {
        &self.info
    }

    fn is_connected(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
/// PCAN USB device
pub struct PcanUsbpacketChannel {
    pub(crate) baud: Option<PCANBaud>,
    pub(crate) use_ext: Option<bool>,
    pub(crate) filter_active: bool,
    dev_handle: PcanUSB,
    driver: PCanDrv,
    device_state: Arc<AtomicBool>,
    open: bool,
}

impl Drop for PcanUsbpacketChannel {
    fn drop(&mut self) {
        self.device_state.store(false, Ordering::Relaxed);
    }
}

impl CanChannel for PcanUsbpacketChannel {
    fn set_can_cfg(&mut self, baud: u32, use_extended: bool) -> ChannelResult<()> {
        let baud_ty = match baud {
            1_000_000 => PCANBaud::Can1Mbps,
            800_000 => PCANBaud::Can800Kbps,
            500_000 => PCANBaud::Can500Kbps,
            250_000 => PCANBaud::Can250Kbps,
            125_000 => PCANBaud::Can125Kbps,
            100_000 => PCANBaud::Can100Kbps,
            95_238 => PCANBaud::Can95Kbps,
            83_333 => PCANBaud::Can83Kbps,
            50_000 => PCANBaud::Can50Kbps,
            47_619 => PCANBaud::Can47Kbps,
            33_333 => PCANBaud::Can33Kbps,
            20_000 => PCANBaud::Can20Kbps,
            10_000 => PCANBaud::Can10Kbps,
            5_000 => PCANBaud::Can5Kbps,
            _ => return Err(ChannelError::ConfigurationError),
        };

        self.baud = Some(baud_ty);
        self.use_ext = Some(use_extended);
        Ok(())
    }
}

impl PacketChannel<CanFrame> for PcanUsbpacketChannel {
    fn open(&mut self) -> ChannelResult<()> {
        if self.open {
            Ok(())
        } else {
            if let Some(b) = self.baud {
                self.driver.initialize_can(self.dev_handle, b)?;
                // Open filter set
                self.open = true;
                Ok(())
            } else {
                Err(ChannelError::ConfigurationError)
            }
        }
    }

    fn close(&mut self) -> ChannelResult<()> {
        if self.open {
            let res = self
                .driver
                .reset_handle(self.dev_handle)
                .map_err(|e| e.into());
            self.open = false;
            res
        } else {
            Ok(())
        }
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, timeout_ms: u32) -> ChannelResult<()> {
        let start = Instant::now();
        for frame in packets {
            self.driver.write(self.dev_handle, frame)?;
            // Write timeout
            if timeout_ms != 0 && start.elapsed().as_millis() > timeout_ms as u128 {
                return Err(ChannelError::WriteTimeout);
            }
        }
        Ok(())
    }

    fn read_packets(&mut self, max: usize, timeout_ms: u32) -> ChannelResult<Vec<CanFrame>> {
        let mut read_packets = vec![];
        let start = Instant::now();
        loop {
            let res = self.driver.read(self.dev_handle);
            match res {
                Ok(f) => {
                    read_packets.push(f);
                }
                Err(ChannelError::BufferEmpty) => return Ok(read_packets),
                Err(e) => return Err(e),
            }
            if read_packets.len() == max {
                return Ok(read_packets);
            }
            if timeout_ms != 0 && start.elapsed().as_millis() > timeout_ms as u128 {
                return if read_packets.len() == 0 {
                    Err(ChannelError::BufferEmpty)
                } else {
                    Ok(read_packets)
                };
            }
        }
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }
}

/// PCAN USB device scanner
#[derive(Debug, Clone)]
pub struct PcanUsbScanner {
    driver: HardwareResult<PCanDrv>,
    cache: Vec<(PcanUSB, HardwareInfo)>,
}

impl Default for PcanUsbScanner {
    fn default() -> Self {
        let mut s = Self {
            driver: PCanDrv::load_lib().map_err(|e| e.into()),
            cache: vec![],
        };
        s.scan_devices();
        s
    }
}

impl PcanUsbScanner {
    fn scan_devices(&mut self) {
        match &self.driver {
            Ok(drv) => {
                let mut res = vec![];
                for dev in ALL_USB_DEVICES {
                    if let Ok((name, version)) = drv.get_device_info(dev) {
                        res.push((
                            *dev,
                            HardwareInfo {
                                name: format!("PCAN-USB_0x{:04X}", *dev as u32),
                                vendor: Some(format!("PEAK-System Technik GmbH")),
                                device_fw_version: Some(name),
                                api_version: Some(version.clone()),
                                library_version: Some(version),
                                library_location: Some(drv.get_path().to_string()),
                                capabilities: HardwareCapabilities {
                                    iso_tp: IsoTpChannelType::Emulated,
                                    can: true,
                                    kline: false,
                                    kline_kwp: false,
                                    sae_j1850: false,
                                    sci: false,
                                    ip: false,
                                },
                            },
                        ))
                    }
                }
                self.cache = res;
            }
            Err(_) => self.cache = vec![],
        }
    }

    fn open_handle(&self, handle: &PcanUSB, info: HardwareInfo) -> HardwareResult<PcanUsbDevice> {
        PcanUsbDevice::new(*handle, info, self.driver.as_ref().unwrap().clone())
    }
}

impl HardwareScanner<PcanUsbDevice> for PcanUsbScanner {
    fn list_devices(&self) -> Vec<HardwareInfo> {
        self.cache.iter().map(|x| x.1.clone()).collect()
    }

    fn open_device_by_index(&self, idx: usize) -> HardwareResult<PcanUsbDevice> {
        match self.cache.get(idx) {
            Some((handle, info)) => {
                return self.open_handle(&handle, info.clone());
            }
            None => Err(HardwareError::DeviceNotFound),
        }
    }

    fn open_device_by_name(&self, name: &str) -> HardwareResult<PcanUsbDevice> {
        for (handle, info) in &self.cache {
            if info.name == name {
                return self.open_handle(&handle, info.clone());
            }
        }
        return Err(HardwareError::DeviceNotFound);
    }
}
