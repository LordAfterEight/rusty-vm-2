mod register;
mod logging;
mod vmerror;

fn main() -> Result<(), vmerror::VmError> {
    let mut register = register::Register::<u8>::create("GPR");

    match register.load(250) {
        Ok(()) => logging::info(&format!("Successfully loaded value into {}\n", register.name)),
        Err(e) => logging::info(&format!("{} in line {}\n", e, line!() - 2))
    }

    logging::process();
    Ok(())
}
