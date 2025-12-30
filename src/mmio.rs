pub trait AddressSpace {
    fn read(&self, addr: u32) -> u8;
    fn write(&mut self, addr: u32, value: u8);
}

#[derive(Clone)]
pub struct Bus {
    pub ram: std::sync::Arc<std::sync::RwLock<crate::memory::Memory>>,
    pub regions: Vec<(u32, u32, std::sync::Arc<std::sync::Mutex<dyn AddressSpace + Send>>)> // base, size, device
}

impl Bus {
    pub fn new_empty(size: usize) -> Self {
        Self {
            ram: std::sync::Arc::new(std::sync::RwLock::new(crate::memory::Memory::empty(size))),
            regions: Vec::new()
        }
    }
}

impl AddressSpace for Bus {
    fn read(&self, addr: u32) -> u8 {
        for (base, size, device) in &self.regions {
            if addr >= *base && addr < *base + *size {
                return device.lock().unwrap().read(addr - base);
            }
        }
        self.ram.read().unwrap().read(addr)
    }
    fn write(&mut self, addr: u32, value: u8) {
        for (base, size, device) in &self.regions {
            if addr >= *base && addr < *base + *size {
                device.lock().unwrap().write(addr - base, value);
            }
        }
        self.ram.write().unwrap().write(addr, value);
    }
}
