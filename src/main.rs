mod core;
mod devices;

fn main() -> Result<(), core::runtime::error::VmRuntimeError> {
    let mut bus = core::runtime::bus::Bus::new();
    let mut cpu = core::cpu::CPU::new();
    bus.load_file("output.rvmimg", &mut cpu)?;
    cpu.link_bus(std::sync::Arc::new(bus));
    let mut logging = core::runtime::logging::Logging::init();


    loop {
        core::runtime::logging::process();
        cpu.cores[0].tick().map_err(core::runtime::error::VmRuntimeError::CoreError)?;
    }
    Ok(())
}
