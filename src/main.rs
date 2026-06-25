mod core;
mod devices;

fn main() -> Result<(), core::runtime::error::VmRuntimeError> {
    let bus = std::sync::Arc::new(core::runtime::bus::Bus::new());
    let mut cpu = core::cpu::CPU::new(bus.clone());
    bus.write(0, 255);
    let mut logging = core::runtime::logging::Logging::init();

    println!("{:?}", cpu);

    loop {
        core::runtime::logging::process();
    }
    Ok(())
}