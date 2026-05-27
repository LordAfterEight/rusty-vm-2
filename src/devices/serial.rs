pub struct SerialPort {
    data: u8,
}

impl SerialPort {
    pub fn new() -> Self {
        Self { data: 0 }
    }
}

impl crate::core::runtime::bus::Device for SerialPort {
    fn read(&self, _addr: u64) -> u8 {
        self.data
    }

    fn write(&mut self, _addr: u64, value: u8) {
        self.data = value;
        crate::core::runtime::logging::info(&format!("Serial port received data: {:>03} | 0x{:02X} | '{}'\n", value, value, value as char));
    }
}