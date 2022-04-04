//! Simulation hardware for unit testing diagnostic servers

use std::{collections::{HashMap, VecDeque}, sync::{Arc, RwLock}};

use crate::ChannelError;

#[derive(Debug, Clone)]
pub struct SimulationIsoTpChannel {
    req_resp_map: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    rx_queue: Arc<RwLock<VecDeque<Vec<u8>>>>,
}

impl SimulationIsoTpChannel {
    pub fn new() -> Self {
        Self {
            req_resp_map: Arc::new(RwLock::new(HashMap::new())),
            rx_queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub fn add_response(&mut self, req: &[u8], resp: &[u8]) {
        self.req_resp_map.write().unwrap().insert(req.to_vec(), resp.to_vec());
    }

    pub fn clear_map(&mut self) {
        self.req_resp_map.write().unwrap().clear();
        self.rx_queue.write().unwrap().clear();
    }
}

impl crate::channel::PayloadChannel for SimulationIsoTpChannel {
    fn open(&mut self) -> crate::channel::ChannelResult<()> {
        Ok(())
    }

    fn close(&mut self) -> crate::channel::ChannelResult<()> {
        Ok(())
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> crate::channel::ChannelResult<()> {
        Ok(())
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> crate::channel::ChannelResult<Vec<u8>> {
        if let Some(r) = self.rx_queue.write().unwrap().pop_front() {
            return Ok(r)
        }
        Err(ChannelError::BufferEmpty)
    }

    fn write_bytes(&mut self, addr: u32, buffer: &[u8], timeout_ms: u32) -> crate::channel::ChannelResult<()> {
        if let Some(expected_response) = self.req_resp_map.read().unwrap().get(buffer) {
            self.rx_queue.write().unwrap().push_back(expected_response.to_vec());
        }
        Ok(())
    }

    fn clear_rx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        self.rx_queue.write().unwrap().clear();
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> crate::channel::ChannelResult<()> {
        Ok(())
    }
}

impl super::IsoTPChannel for SimulationIsoTpChannel {
    fn set_iso_tp_cfg(&mut self, cfg: crate::channel::IsoTPSettings) -> crate::channel::ChannelResult<()> {
        Ok(())
    }
}

