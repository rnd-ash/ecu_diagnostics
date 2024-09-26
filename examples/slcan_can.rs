use serial_rs::{FlowControl, SerialPortSettings};
use ecu_diagnostics::{channel::CanFrame, hardware::Hardware};

extern crate ecu_diagnostics;
extern crate serial_rs;

fn main() {
    env_logger::builder()
        .format_timestamp(Some(env_logger::TimestampPrecision::Millis))
        .init();

    let port = serial_rs::new_from_path(
        "/dev/cu.usbmodem90379201".into(),
        Some(
            SerialPortSettings::default()
                .baud(2000000)
                .read_timeout(Some(1))
                .write_timeout(Some(100))
                .set_flow_control(FlowControl::None)
                .set_blocking(true),
        ),
    ).unwrap();

    let mut d = ecu_diagnostics::hardware::slcan::device::SlCanDevice::new(port, 1000);
    
    let mut can = d.create_can_channel().unwrap();

    can.set_can_cfg(83_333, false).unwrap();

    can.open().unwrap();

    let packets = can.read_packets(100, 500).unwrap();

    for p in packets {
        println!("{:02X?}", p);
    }

    can.write_packets(vec![CanFrame::new(0x5B4, vec![2, 0x10, 0x92, 0, 0,0,0].as_ref(), false)], 100).unwrap();

    let packets = can.read_packets(100, 500).unwrap();

    for p in packets {
        println!("{:02X?}", p);
    }
}