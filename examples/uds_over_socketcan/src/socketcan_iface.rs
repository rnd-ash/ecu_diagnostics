use core::time;
use std::{borrow::BorrowMut, cell::RefCell, ops::Deref, time::Instant};

use ecu_diagnostics::channel::{BaseChannel, ChannelError, IsoTPChannel, IsoTPSettings};
use socketcan_isotp::{FlowControlOptions, IsoTpBehaviour, IsoTpOptions, IsoTpSocket, EFF_FLAG};

// Convert SocketCAN errors into Channel Errors

// Our socket CAN Interface over an ISO TP Channel

pub struct SocketCANInterface {
    iface_name: String,
    iface: Option<IsoTpSocket>,
    opts: Option<IsoTPSettings>,
    send_id: u32,
    recv_id: u32,
}

impl SocketCANInterface {
    pub fn new(iface_name: String) -> Self {
        Self {
            iface_name,
            iface: None,
            opts: None,
            send_id: 0,
            recv_id: 0,
        }
    }
}

// Base channel implementation for ISOTP
impl BaseChannel for SocketCANInterface {
    fn open(&mut self) -> ecu_diagnostics::channel::ChannelResult<()> {
        if self.opts.is_none() {
            return Err(ChannelError::APIError {
                api_name: "SocketCAN".into(),
                code: 0,
                desc: "IsoTP configuration is null".into(),
            });
        }
        if self.iface.is_some() {
            return Err(ChannelError::APIError {
                api_name: "SocketCAN".into(),
                code: 1,
                desc: "SocketCAN interface is already open".into(),
            });
        }

        let fc_options = FlowControlOptions::default();

        let mut isotp_options = IsoTpOptions::default();
        // Set frame pad byte to 0x00
        isotp_options.set_rxpad_content(0x00);
        isotp_options.set_txpad_content(0x00);

        let mut flags: IsoTpBehaviour = IsoTpBehaviour::empty();

        if self.opts.unwrap().extended_addressing {
            // Extended addressing
            flags |= IsoTpBehaviour::CAN_ISOTP_EXTEND_ADDR
        }
        if self.opts.unwrap().pad_frame {
            // Pad frame flag
            flags |= IsoTpBehaviour::CAN_ISOTP_RX_PADDING | IsoTpBehaviour::CAN_ISOTP_TX_PADDING;
        }

        isotp_options.set_flags(flags);

        match IsoTpSocket::open_with_opts(
            &self.iface_name,
            self.recv_id,
            self.send_id,
            Some(isotp_options),
            Some(fc_options),
            None,
        ) {
            Ok(channel) => {
                channel.set_nonblocking(true);
                self.iface = Some(channel);
                Ok(())
            }
            Err(e) => {
                return Err(ChannelError::APIError {
                    api_name: "SocketCAN".into(),
                    code: 1,
                    desc: format!("{}", e),
                })
            }
        }
    }

    fn close(&mut self) -> ecu_diagnostics::channel::ChannelResult<()> {
        self.iface.take();
        Ok(())
    }

    fn set_ids(&mut self, send: u32, recv: u32) -> ecu_diagnostics::channel::ChannelResult<()> {
        self.send_id = send;
        self.recv_id = recv;
        Ok(())
    }

    fn read_bytes(&mut self, timeout_ms: u32) -> ecu_diagnostics::channel::ChannelResult<Vec<u8>> {
        match self.iface.as_mut() {
            Some(iface) => {
                if timeout_ms == 0 {
                    // 1 read attempt (Whatever is in the Rx buffer gets read)
                    if let Ok(resp) = iface.read() {
                        println!("{:02X?}", resp);
                        return Ok(resp.to_vec());
                    } else {
                        return Err(ChannelError::BufferEmpty);
                    }
                } else {
                    // Loop read
                    let start = Instant::now();
                    while start.elapsed().as_millis() <= timeout_ms as u128 {
                        if let Ok(resp) = iface.read() {
                            return Ok(resp.to_vec());
                        }
                    }
                    return Err(ChannelError::ReadTimeout);
                }
            }
            None => Err(ChannelError::InterfaceNotOpen),
        }
    }

    fn write_bytes(
        &mut self,
        addr: u32,
        buffer: &[u8],
        timeout_ms: u32,
    ) -> ecu_diagnostics::channel::ChannelResult<()> {
        match self.iface.as_mut() {
            Some(iface) => {
                iface.write(buffer);
                Ok(())
            }
            None => Err(ChannelError::InterfaceNotOpen),
        }
    }

    fn clear_rx_buffer(&mut self) -> ecu_diagnostics::channel::ChannelResult<()> {
        match self.iface.as_mut() {
            Some(iface) => {
                while iface.read().is_ok() {}
                Ok(())
            }
            None => Err(ChannelError::InterfaceNotOpen),
        }
    }

    fn clear_tx_buffer(&mut self) -> ecu_diagnostics::channel::ChannelResult<()> {
        Ok(())
    }
}

// ISO TP Channel configuration for SocketCAN
impl IsoTPChannel for SocketCANInterface {
    fn set_iso_tp_cfg(
        &mut self,
        cfg: IsoTPSettings,
    ) -> ecu_diagnostics::channel::ChannelResult<()> {
        self.opts = Some(cfg);
        Ok(())
    }
}
