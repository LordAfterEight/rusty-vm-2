pub struct SerialPort {
    data: std::cell::Cell<u8>,
}

impl SerialPort {
    pub fn new() -> Self {
        Self { data: std::cell::Cell::new(0) }
    }
}

impl crate::core::runtime::bus::Device for SerialPort {
    fn read8(&self, _addr: u64) -> u8 {
        self.data.get()
    }

    fn write8(&self, _addr: u64, value: u8) {
        self.data.set(value);
        crate::core::runtime::logging::info(&format!("Serial port received data: {:>03} | 0x{:02X} | '{}'\n", value, value, value as char));
    }
}
