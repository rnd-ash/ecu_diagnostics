//! VW Transport protocol 2
//! 
//! This piggy-backs off an open CAN Channel, to provide a PayloadChannel
//! that can be used by the Diagnostic server

use std::{sync::{Arc, atomic::AtomicBool, mpsc}, thread::JoinHandle, time::Duration};

use crate::channel::{CanChannel, ChannelError, ChannelResult, PayloadChannel, VwTp2Settings};

#[derive(Clone)]
pub enum VwChannelCmd {
    SetIds(u32, u32),
    Open,
    Close,
    Write(Vec<u8>)
}

pub enum ChannelState {
    None,
    Init,
    Open(u32, u32)
}

pub struct VwTp2 {
    running: Arc<AtomicBool>,
    handle: JoinHandle<Box<dyn CanChannel>>,
    tx_cmd: mpsc::Sender<VwChannelCmd>
}

impl VwTp2 {
    pub fn new(mut can: Box<dyn CanChannel>, ecu_id: u8, settings: VwTp2Settings) -> ChannelResult<Self> {
        if can.is_open() {
            Err(ChannelError::ConfigurationError)
        } else {
            let (channel_cmd_tx, channel_cmd_rx) = mpsc::channel();
            let killswitch = Arc::new(AtomicBool::new(true));
            let killswitch_c = killswitch.clone();

            // Open CAN Channel first
            // VW TP never uses EXT CAN Addr
            can.set_can_cfg(settings.can_baud, false);
            can.open()?;

            let handle = std::thread::spawn(move|| {
                let state = ChannelState::None;


                if let Ok(tester_req) = channel_cmd_rx.try_recv() {
                    
                }

                while killswitch_c.load(std::sync::atomic::Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(10));
                }
                let _ = can.close();
                can
            });

            Ok(Self {
                running: killswitch,
                handle: handle,
                tx_cmd: channel_cmd_tx,
            })
        }
    }

    pub fn release(self) -> Box<dyn CanChannel> {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        self.handle.join().unwrap()
    }
}

impl PayloadChannel for VwTp2 {
    fn open(&mut self) -> ChannelResult<()> {
        self.tx_cmd.send(VwChannelCmd::Open)?;
        Ok(())
    }

    fn close(&mut self) -> ChannelResult<()> {
        self.tx_cmd.send(VwChannelCmd::Close)?;
        Ok(())
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ChannelResult<()> {
        let valid_range = 0x300..=0x310u32;
        if !valid_range.contains(&send) || !valid_range.contains(&recv) || send == recv {
            Err(ChannelError::ConfigurationError)
        } else {
            self.tx_cmd.send(VwChannelCmd::SetIds(send, recv))?;
            Ok(())
        }
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        todo!()
    }

    fn write_bytes(
        &mut self,
        _addr: u32,
        _ext_id: Option<u8>,
        buffer: &[u8],
        _timeout_ms: u32,
    ) -> ChannelResult<()> {
        self.tx_cmd.send(VwChannelCmd::Write(buffer.to_vec()))?;
        Ok(())
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
        Ok(())
    }
}

#[cfg(test)]
pub mod vw_tp_test {
    use crate::{channel::{PayloadChannel, VwTp2Settings}, hardware::{Hardware, HardwareScanner, socketcan::SocketCanScanner, vwtp::VwTp2}};

    #[test]
    pub fn test_vwtp() {
        let mut hw = SocketCanScanner::new().open_device_by_name("can0").unwrap();
        let mut can = hw.create_can_channel().unwrap();
        let vw_settings = VwTp2Settings::default();
        let mut vwtp = VwTp2::new(can, 0x0A, vw_settings).unwrap();
        vwtp.set_ids(0x300, 0x308).unwrap();
        vwtp.open().unwrap();
        vwtp.write_bytes(0x300, None, &[0x10, 0x92], 1000).unwrap();
        println!("{:?}", vwtp.read_bytes(1000));
    }
}