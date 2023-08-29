use std::{
    sync::{mpsc, Arc},
    time::Instant,
};

use ecu_diagnostics::{
    channel::{
        CanChannel, CanFrame, ChannelError, IsoTPChannel, IsoTPSettings, PacketChannel,
        PayloadChannel,
    },
    hardware::software_isotp::SoftwareIsoTpChannel,
};

pub struct EmuCanChannel {
    name: &'static str,
    in_queue: Arc<mpsc::Receiver<CanFrame>>,
    out_queue: mpsc::Sender<CanFrame>,
}

unsafe impl Send for EmuCanChannel {}
unsafe impl Sync for EmuCanChannel {}

impl EmuCanChannel {
    pub fn new(
        sender: mpsc::Sender<CanFrame>,
        receiver: mpsc::Receiver<CanFrame>,
        name: &'static str,
    ) -> Self {
        Self {
            name,
            in_queue: Arc::new(receiver),
            out_queue: sender,
        }
    }
}

impl CanChannel for EmuCanChannel {
    fn set_can_cfg(
        &mut self,
        _baud: u32,
        _use_extended: bool,
    ) -> ecu_diagnostics::channel::ChannelResult<()> {
        Ok(())
    }
}

impl PacketChannel<CanFrame> for EmuCanChannel {
    fn open(&mut self) -> ecu_diagnostics::channel::ChannelResult<()> {
        Ok(())
    }

    fn close(&mut self) -> ecu_diagnostics::channel::ChannelResult<()> {
        Ok(())
    }

    fn write_packets(
        &mut self,
        packets: Vec<CanFrame>,
        _timeout_ms: u32,
    ) -> ecu_diagnostics::channel::ChannelResult<()> {
        for p in packets {
            log::debug!("{} Out -> {p:02X?}", self.name);
            self.out_queue.send(p).unwrap();
        }
        Ok(())
    }

    fn read_packets(
        &mut self,
        max: usize,
        timeout_ms: u32,
    ) -> ecu_diagnostics::channel::ChannelResult<Vec<CanFrame>> {
        let mut read_packets = vec![];
        let start = Instant::now();
        loop {
            let res = self
                .in_queue
                .try_recv()
                .map_err(|_| ChannelError::BufferEmpty);
            match res {
                Ok(f) => {
                    log::debug!("{} In  -> {f:02X?}", self.name);
                    read_packets.push(f);
                }
                Err(ChannelError::BufferEmpty) => return Ok(read_packets),
                Err(e) => return Err(e),
            }
            if read_packets.len() == max {
                return Ok(read_packets);
            }
            if timeout_ms != 0 && start.elapsed().as_millis() > timeout_ms as u128 {
                return if read_packets.is_empty() {
                    Err(ChannelError::BufferEmpty)
                } else {
                    Ok(read_packets)
                };
            }
        }
    }

    fn clear_rx_buffer(&mut self) -> ecu_diagnostics::channel::ChannelResult<()> {
        while self.in_queue.try_recv().is_ok() {}
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> ecu_diagnostics::channel::ChannelResult<()> {
        Ok(())
    }
}

fn setup(
    bs: u8,
    stmin: u8,
    padding: bool,
    ext_address: Option<(u8, u8)>,
    ecu1_addr: u32,
    ecu2_addr: u32,
) -> (SoftwareIsoTpChannel, SoftwareIsoTpChannel) {
    let (ecu1tx, ecu1rx) = mpsc::channel::<CanFrame>();
    let (ecu2tx, ecu2rx) = mpsc::channel::<CanFrame>();
    let ecu1 = Box::new(EmuCanChannel::new(ecu2tx, ecu1rx, "Tester"));
    let ecu2 = Box::new(EmuCanChannel::new(ecu1tx, ecu2rx, "ECU"));

    let mut iso_tp1 = SoftwareIsoTpChannel::new(ecu1);
    let mut iso_tp2 = SoftwareIsoTpChannel::new(ecu2);

    iso_tp1.set_iso_tp_cfg(IsoTPSettings {
        block_size: bs,
        st_min: stmin,
        extended_addresses: ext_address,
        pad_frame: padding,
        can_speed: 500_000,
        can_use_ext_addr: false,
    });

    iso_tp2.set_iso_tp_cfg(IsoTPSettings {
        block_size: bs,
        st_min: stmin,
        extended_addresses: ext_address,
        pad_frame: padding,
        can_speed: 500_000,
        can_use_ext_addr: false,
    });

    iso_tp1.set_ids(ecu1_addr, ecu2_addr);
    iso_tp2.set_ids(ecu2_addr, ecu1_addr);

    PayloadChannel::open(&mut iso_tp1);
    PayloadChannel::open(&mut iso_tp2);

    (iso_tp1, iso_tp2)
}
/*
#[test]
fn test_single_frame() {
    env_logger::try_init();
    let TX_BYTES = &[0x01, 0x02, 0x03, 0x04, 0x05];

    let (mut ch1, mut ch2) = setup(8, 20, true, None, 0x07E1, 0x07E9);

    ch1.write_bytes(0x07E1, None, TX_BYTES, 0).expect("Write failed!");

    let r = ch2.read_bytes(1000);
    assert!(r.is_ok());
    assert_eq!(TX_BYTES.to_vec(), r.unwrap());
}
*/
#[test]
fn test_multi_frame() {
    env_logger::try_init();
    let TX_BYTES = (0..0xFF).collect::<Vec<u8>>();

    let (mut ch1, mut ch2) = setup(8, 20, true, None, 0x07E1, 0x07E9);

    ch1.write_bytes(0x07E1, None, &TX_BYTES, 5000)
        .expect("Write failed!");

    let mut r = ch2.read_bytes(5000);

    assert!(r.is_ok());
    assert_eq!(TX_BYTES.to_vec(), r.unwrap());

    let TX2_BYTES = (0x00..0xFF).rev().collect::<Vec<u8>>();

    ch1.write_bytes(0x07E1, None, &TX2_BYTES, 5000)
        .expect("Write failed!");

    r = ch2.read_bytes(5000);

    assert!(r.is_ok());
    assert_eq!(TX2_BYTES.to_vec(), r.unwrap());
}
