#[derive(Debug)]
pub struct Bus {
    memory: std::sync::Arc<std::sync::RwLock<Vec<u8>>>,
    devices: Vec<MemoryRegion>
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: std::sync::Arc::new(std::sync::RwLock::new(vec![0; 1024 * 1024 * 1024 * 4 / (1024*1024)*4])),
            devices: Vec::new(),
        }
    }
    
    pub fn read(&self, addr: u64) -> u8 {
        for region in &self.devices {
            if addr >= region.start && addr <= region.end {
                return region.device.read(addr);
            }
        }
        self.memory.read().unwrap()[addr as usize]
    }

    pub fn write(&self, addr: u64, value: u8) {
        for region in &self.devices {
            if addr >= region.start && addr <= region.end {
                region.device.write(addr, value);
                return;
            }
        }
        self.memory.write().unwrap()[addr as usize] = value;
    }

    pub fn map_device(&mut self, start: u64, end: u64, device: Box<dyn Device>) {
        self.devices.push(MemoryRegion { start, end, device });
    }
}

#[derive(Debug)]
pub struct MemoryRegion {
    start: u64,
    end: u64,
    device: Box<dyn Device>
}

pub trait Device {
    fn read(&self, addr: u64) -> u8;
    fn write(&self, addr: u64, value: u8); // &mut self → &self
}

impl std::fmt::Debug for dyn Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}