mod register;
mod logging;
mod vmerror;

fn main() -> Result<logging::Message, vmerror::VmError> {
    let mut register = register::Register::<u8>::create("GPR");
    let mut logging = crate::logging::Logging::init();
    logging.info("Testing INFO message");
    logging.debug("Testing DEBUG message");
    logging.warn("Testing WARN message");
    logging.error("Testing ERROR message");

    logging.process();
    Ok(logging::Message::info("Done"))
}
