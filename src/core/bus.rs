pub struct Bus {
    memory: Box<std::sync::Arc<std::sync::Mutex<[u8; 0x10000]>>>,
    devices: Vec<MemoryRegion>
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: Box::new(std::sync::Arc::new(std::sync::Mutex::new([0; 0x10000]))),
            devices: Vec::new(),
        }
    }
    
    pub fn read(&self, addr: u64) -> u8 {
        for region in &self.devices {
            if addr >= region.start && addr <= region.end {
                return region.device.read(addr);
            }
        }
        self.memory.lock().unwrap()[addr as usize]
    }
    
    pub fn write(&mut self, addr: u64, value: u8) {
        for region in &mut self.devices {
            if addr >= region.start && addr <= region.end {
                region.device.write(addr, value);
                return;
            }
        }
        self.memory.lock().unwrap()[addr as usize] = value;
    }

    pub fn map_device(&mut self, start: u64, end: u64, device: Box<dyn Device>) {
        self.devices.push(MemoryRegion { start, end, device });
    }
}

pub struct MemoryRegion {
    start: u64,
    end: u64,
    device: Box<dyn Device>
}

pub trait Device {
    fn read(&self, addr: u64) -> u8;
    fn write(&mut self, addr: u64, value: u8);
}