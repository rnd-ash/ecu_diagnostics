use ecu_diagnostics::{DiagError, DiagServerResult, uds::{UDSCommand, UdsDiagnosticServer, UdsServerOptions, UdsVoidHandler},
channel::{BaseChannel, ChannelError, ChannelResult, IsoTPChannel, IsoTPSettings}};

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
    fn open_iso_tp(&mut self, cfg: IsoTPSettings) -> ChannelResult<()> {
        println!(
            "IsoTPChannel: configure_iso_tp Called. BS: {}, ST-MIN: {}",
            cfg.block_size, cfg.st_min
        );
        Ok(())
    }
}

impl<T: 'static + Clone + Fn(&[u8]) -> Option<Vec<u8>>> BaseChannel for UdsSimEcu<T> {

    fn set_ids(&mut self, send: u32, recv: u32, global_tp_id: Option<u32>) -> ChannelResult<()> {
        println!(
            "BaseChannel: set_ids Called. send: {}, recv: {}, global_tp_id: {:?}",
            send, recv, global_tp_id
        );
        Ok(())
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ChannelResult<Vec<u8>> {
        println!("BaseChannel: read_bytes Called. timeout_ms: {}", timeout_ms);
        if self.out_buffer.is_empty() {
            println!("-- NOTHING TO SEND");
            Err(ChannelError::BufferEmpty)
        } else {
            let send = self.out_buffer[0].clone();
            println!("-- Sending {:02X?} back to diag server", &send);
            self.out_buffer.drain(0..1);
            Ok(send)
        }
    }

    fn write_bytes(&mut self, buffer: &[u8], timeout_ms: u32) -> ChannelResult<()> {
        println!(
            "BaseChannel: write_bytes Called. Tx: {:02X?}, timeout_ms: {}",
            buffer, timeout_ms
        );
        if let Some(sim_resp) = (self.on_data_callback)(buffer) {
            self.out_buffer.push(sim_resp);
        }
        Ok(())
    }

    fn clear_rx_buffer(&mut self) -> ChannelResult<()> {
        self.out_buffer = Vec::new();
        Ok(())
    }

    fn clear_tx_buffer(&mut self) -> ChannelResult<()> {
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
        send_id: 0x07E0,
        recv_id: 0x07E8,
        read_timeout_ms: 1000,
        write_timeout_ms: 1000,
        global_tp_id: None,
        tester_present_interval_ms: 2000,
        tester_present_require_response: true,
    };

    let mut server = UdsDiagnosticServer::new_over_iso_tp(
        settings,
        sim_ecu,
        IsoTPSettings {
            block_size: 8,
            st_min: 20,
            extended_addressing: false,
            pad_frame: true,
            can_speed: 500000,
            can_use_ext_addr: false
        },
        UdsVoidHandler,
    )
    .unwrap();

    server
        .execute_command_with_response(UDSCommand::DiagnosticSessionControl, &[0x10])
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5000));
}
