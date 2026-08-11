use crate::core::runtime::logging::info;

pub struct SerialPort {
    data: std::cell::Cell<u8>,
    data_arr: std::cell::RefCell<String>,
}

impl SerialPort {
    pub fn new() -> Self {
        Self {
            data: std::cell::Cell::new(0),
            data_arr: std::cell::RefCell::new(String::new()),
        }
    }
}

impl crate::core::runtime::bus::Device for SerialPort {
    fn read8(&self, _addr: u64) -> u8 {
        self.data.get()
    }

    fn write8(&self, addr: u64, value: u8) {
        if addr == 1 && value == 1 {
            info(&format!("Serial Port Buffer: {}", self.data_arr.borrow()));
        } else if addr == 0 {
            self.data.set(value);
            self.data_arr.borrow_mut().push(value as char);
            crate::core::runtime::logging::info(&format!(
                "Serial port received data: {:>03} | 0x{:02X} | '{}'",
                value, value, value as char
            ));
        }
    }
}
