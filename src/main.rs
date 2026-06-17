mod core;
mod devices;

fn main() -> Result<(), core::runtime::error::VmRuntimeError> {
    let mut bus = core::runtime::bus::Bus::new();
    let mut logging = core::runtime::logging::Logging::init();
    let mut core = core::cpu::core::Core::new("Core", &bus);
    core.registers[0].load(0xFFFFFFFF1u64)?;

    core::runtime::logging::process();
    Ok(())
}