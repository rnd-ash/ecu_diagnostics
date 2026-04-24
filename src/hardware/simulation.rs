//! Simulation hardware for unit testing diagnostic servers

use std::{sync::{Mutex, mpsc}};

use crate::{channel::{CanChannel, CanFrame, PacketChannel}};

pub (crate) struct SimulationCanChannel {
    pub name: &'static str,
    pub rx: Mutex<mpsc::Receiver<CanFrame>>,
    pub tx: mpsc::Sender<CanFrame>,
    pub opened: bool
}


impl PacketChannel<CanFrame> for SimulationCanChannel {
    fn open(&mut self) -> crate::channel::ChannelResult<()> {
        self.opened = true;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.opened
    }

    fn close(&mut self) -> crate::channel::ChannelResult<()> {
        self.opened = false;
        Ok(())
    }

    fn write_packets(&mut self, packets: Vec<CanFrame>, timeout_ms: u32) -> crate::channel::ChannelResult<()> {
        for p in packets {
            log::debug!("{} OUT: {:02X?}", self.name, p);
            self.tx.send(p).unwrap();
        }
        Ok(())
    }

    fn read_packets(&mut self, _max: usize, _timeout_ms: u32) -> crate::channel::ChannelResult<Vec<CanFrame>> {
        let mut v = Vec::new();
        while let Ok(p) = self.rx.lock().unwrap().try_recv() {
            log::debug!("{}  IN: {:02X?}", self.name, p);
            v.push(p);
        }
        Ok(v)
    }

    fn clear_rx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        while self.rx.lock().unwrap().try_recv().is_ok(){};
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        Ok(())
    }
}

impl CanChannel for SimulationCanChannel {
    fn set_can_cfg(&mut self, _baud: u32, _use_extended: bool) -> crate::channel::ChannelResult<()> {
        Ok(())
    }
}

pub (crate) fn new_sim_can_pair() -> (SimulationCanChannel, SimulationCanChannel) {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    (
        SimulationCanChannel {
            name: "SIMCAN0",
            rx: Mutex::new(rx1),
            tx: tx2,
            opened: false,
        },
        SimulationCanChannel {
            name: "SIMCAN1",
            rx: Mutex::new(rx2),
            tx: tx1,
            opened: false,
        }
    )
}