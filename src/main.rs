mod core;
mod devices;

fn main() -> Result<(), core::vmerror::VmError> {
    let mut bus = core::bus::Bus::new();
    let mut logging = core::logging::Logging::init();

    let port = devices::serial::SerialPort::new();
    bus.map_device(0xFF00, 0xFF01, Box::new(port));

    bus.write(0xFF00, b'H');
    
    core::logging::process();
    Ok(())
}