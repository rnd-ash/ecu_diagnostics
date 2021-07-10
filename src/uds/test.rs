use super::*;

#[derive(Clone)]
pub struct UdsSimEcu<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> {
    on_data_callback: T,
    out_buffer: Vec<Vec<u8>>,
}
unsafe impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> Send for UdsSimEcu<T> {}
unsafe impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> Sync for UdsSimEcu<T> {}

impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> UdsSimEcu<T> {
    pub fn new(on_data_callback: T) -> Self {
        Self {
            on_data_callback,
            out_buffer: Vec::new(),
        }
    }

    pub fn set_callback(&mut self, on_data_callback: T) {
        self.on_data_callback = on_data_callback
    }
}

impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> IsoTPChannel for UdsSimEcu<T> {
    fn configure_iso_tp(&mut self, cfg: IsoTPSettings) -> DiagServerResult<()> {
        println!(
            "IsoTPChannel: configure_iso_tp Called. BS: {}, ST-MIN: {}",
            cfg.block_size, cfg.st_min
        );
        Ok(())
    }

    fn clone_isotp(&self) -> Box<dyn IsoTPChannel> {
        println!("IsoTPChannel: clone_isotp Called");
        Box::new(self.clone())
    }

    fn into_base(&self) -> Box<dyn BaseChannel> {
        println!("IsoTPChannel: into_base Called");
        Box::new(self.clone())
    }
}

impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> BaseChannel for UdsSimEcu<T> {
    fn clone_base(&self) -> Box<dyn BaseChannel> {
        println!("BaseChannel: into_base Called");
        Box::new(self.clone())
    }

    fn set_baud(&mut self, baud: u32) -> DiagServerResult<()> {
        println!("BaseChannel: set_baud Called. Baud: {} bps", baud);
        Ok(())
    }

    fn set_ids(&mut self, send: u32, recv: u32, global_tp_id: Option<u32>) -> DiagServerResult<()> {
        println!(
            "BaseChannel: set_ids Called. send: {}, recv: {}, global_tp_id: {:?}",
            send, recv, global_tp_id
        );
        Ok(())
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> DiagServerResult<Vec<u8>> {
        println!("BaseChannel: read_bytes Called. timeout_ms: {}", timeout_ms);
        if self.out_buffer.is_empty() {
            println!("-- NOTHING TO SEND");
            Err(DiagError::Timeout)
        } else {
            let send = self.out_buffer[0].clone();
            println!("-- Sending {:02X?} back to diag server", &send);
            self.out_buffer.drain(0..1);
            Ok(send)
        }
    }

    fn write_bytes(&mut self, buffer: &[u8], timeout_ms: u32) -> DiagServerResult<()> {
        println!(
            "BaseChannel: write_bytes Called. Tx: {:02X?}, timeout_ms: {}",
            buffer, timeout_ms
        );
        if let Some(sim_resp) = (self.on_data_callback)(buffer) {
            self.out_buffer.push(sim_resp);
        }
        Ok(())
    }

    fn clear_rx_buffer(&mut self) -> DiagServerResult<()> {
        self.out_buffer = Vec::new();
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> DiagServerResult<()> {
        Ok(())
    }
}

#[test]
pub fn test_send_uds_cmd() {
    fn callback(buf: &[u8]) -> Option<Vec<u8>> {
        if buf[0] == 0x10 {
            // Start ID
            return Some(vec![0x50, buf[1]]);
        } else {
            None
        }
    }

    let sim_ecu = UdsSimEcu::new(callback);

    let settings = UdsServerOptions {
        baud: 500000,
        send_id: 0x07E0,
        recv_id: 0x07E8,
        read_timeout_ms: 1000,
        write_timeout_ms: 1000,
        global_tp_id: None,
        tester_present_interval_ms: 2000,
        server_refresh_interval_ms: 10,
        tester_present_require_response: true,
    };

    let mut server = UdsDiagnosticServer::new_over_iso_tp(
        settings,
        Box::new(sim_ecu),
        IsoTPSettings {
            block_size: 8,
            st_min: 20,
            extended_addressing: false,
            pad_frame: true,
        },
        None,
    )
    .unwrap();

    server
        .execute_command_with_response(UDSCommand::DiagnosticSessionControl, &[0x10])
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5000));
}
