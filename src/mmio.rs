pub trait AddressSpace {
    fn read8(&self, addr: u32) -> u8;
    fn write8(&mut self, addr: u32, value: u8);
    fn write32(&mut self, addr: u32, value: u32);
}

#[derive(Clone)]
pub struct MmioRegion {
    pub name: String,
    pub base: u32,
    pub size: u32,
    pub device: std::sync::Arc<std::sync::Mutex<dyn AddressSpace + Send>>
}

#[derive(Clone)]
pub struct Bus {
    pub ram: std::sync::Arc<std::sync::RwLock<crate::memory::Memory>>,
    pub regions: Vec<MmioRegion>
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
    fn read8(&self, addr: u32) -> u8 {
        for device in &self.regions {
            if addr >= device.base && addr < device.base + device.size {
                info!("Reading from device {}", device.name);
                return device.device.lock().unwrap().read8(addr - device.base);
            }
        }
        self.ram.read().unwrap().read8(addr)
    }
    fn write8(&mut self, addr: u32, value: u8) {
        info!("Writing value {} to address {}", value, addr);
        for device in &self.regions {
            if addr >= device.base && addr < device.base + device.size {
                info!("Forwarding to device {} at address {}...", device.name, addr);
                device.device.lock().unwrap().write8(addr - device.base, value);
                info!("Done");
                return;
            }
        }
        self.ram.write().unwrap().write8(addr, value);
    }

    fn write32(&mut self, addr: u32, value: u32) {
        info!("Writing value {} to address {}", value, addr);
        for device in &self.regions {
            if addr >= device.base && addr < device.base + device.size {
                info!("Forwarding to device {} at address {}...", device.name, addr);
                device.device.lock().unwrap().write32(addr - device.base, value);
                info!("Done");
                return;
            }
        }
        self.ram.write().unwrap().write32(addr, value);
    }
}
