// Demo of the usage of PEAK API for PEAK hardware

/**
 * NOTICE:
 * 
 * This example appears to crash when compiled for i686-pc-windows-msvc target
 * The x86_64-pc-windows-msvc will work fine.
 * 
 * Test:
 * 32bit (SEGFAULT):
 * ```
 * $env:RUST_LOG="debug"
 * cargo run --example peak --target i686-pc-windows-msvc
 * ```
 * 
 * 64bit (Works OK):
 * ```
 * $env:RUST_LOG="debug"
 * cargo run --example peak --target x86_64-pc-windows-msvc
 * ```
 * 
 * Secondary notice:
 * For some reason, running the 64bit version twice (Or after the 32bit version crashes),
 * PEAK will not detect any incomming CAN packets.
 */

#[cfg(windows)]
fn main() {
    env_logger::init();
    use ecu_diagnostics::hardware::{self, HardwareScanner, Hardware};

    let scanner = hardware::pcan_usb::PcanUsbScanner::default();
    println!("Available PEAK devices:");
    for dev in scanner.list_devices() {
        println!("{}", dev.name);
    }
    let mut device = scanner.open_device_by_index(0).unwrap();
    // Create a CAN channel
    let mut can = device.create_can_channel().unwrap();
    can.set_can_cfg(500_000, false).expect("Failed to configure PEAK device CAN");
    can.open().expect("Failed to open CAN channel");
    loop {
        for packet in can.read_packets(10, 0).unwrap_or_default() {
            println!("Incomming CAN packet: {:02X?}", packet)
        }
    }
}