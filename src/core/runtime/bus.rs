#[derive(Debug)]
pub struct Bus {
    memory: std::sync::Arc<std::sync::RwLock<Vec<u8>>>,
    devices: Vec<MemoryRegion>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            memory: std::sync::Arc::new(std::sync::RwLock::new(vec![0; 1024 * 1024 * 1024 * 4])),
            devices: Vec::new(),
        }
    }

    pub fn read8(&self, addr: u64) -> u8 {
        for region in &self.devices {
            if addr >= region.start && addr <= region.end {
                return region.device.read8(addr);
            }
        }
        self.memory.read().unwrap()[addr as usize]
    }

    pub fn read16(&self, addr: u64) -> u16 {
        for region in &self.devices {
            if addr >= region.start && addr <= region.end {
                return region.device.read16(addr);
            }
        }
        let memory = self.memory.read().unwrap();
        u16::from_le_bytes([memory[addr as usize], memory[addr as usize + 1]])
    }

    pub fn read32(&self, addr: u64) -> u32 {
        for region in &self.devices {
            if addr >= region.start && addr <= region.end {
                return region.device.read32(addr);
            }
        }
        let memory = self.memory.read().unwrap();
        u32::from_le_bytes([
            memory[addr as usize],
            memory[addr as usize + 1],
            memory[addr as usize + 2],
            memory[addr as usize + 3],
        ])
    }

    pub fn write8(&self, addr: u64, value: u8) {
        for region in &self.devices {
            if addr >= region.start && addr <= region.end {
                region.device.write8(addr, value);
                return;
            }
        }
        self.memory.write().unwrap()[addr as usize] = value;
    }

    pub fn write16(&self, addr: u64, value: u16) {
        for region in &self.devices {
            if addr >= region.start && addr <= region.end {
                region.device.write16(addr, value);
                return;
            }
        }
        let bytes = value.to_le_bytes();
        let mut memory = self.memory.write().unwrap();
        memory[addr as usize] = bytes[0];
        memory[addr as usize + 1] = bytes[1];
    }

    pub fn write32(&self, addr: u64, value: u32) {
        for region in &self.devices {
            if addr >= region.start && addr <= region.end {
                region.device.write32(addr, value);
                return;
            }
        }
        let bytes = value.to_le_bytes();
        let mut memory = self.memory.write().unwrap();
        memory[addr as usize] = bytes[0];
        memory[addr as usize + 1] = bytes[1];
        memory[addr as usize + 2] = bytes[2];
        memory[addr as usize + 3] = bytes[3];
    }

    pub fn map_device(&mut self, start: u64, end: u64, device: Box<dyn Device>) {
        self.devices.push(MemoryRegion { start, end, device });
    }

    pub fn load_file(
        &mut self,
        path: &str,
        cpu: &mut crate::core::cpu::CPU,
    ) -> Result<(), FileLoadError> {
        let raw = std::fs::read(path)?;
        let mut offset = 0x44;
        let mut blk_counter = 0;
        let descriptor = String::from_utf8_lossy(&raw[0x0..=0xF]);
        let header = String::from_utf8_lossy(&raw[0x10..=0x3F]);
        let block_amount = u32::from_le_bytes(*raw.get(0x40..=0x43).and_then(|bytes| bytes.as_array()).ok_or(FileLoadError::InvalidImage)?);
        println!(
            "\n\nDECODING\n\nRaw Descriptor:     {}\nRaw Header:         {}\nAmount Of Blocks:   {}\n",
            descriptor, header, block_amount
        );
        let mut start_block = (false, 0);
        while blk_counter < block_amount {
            println!("\nBLOCK {}", blk_counter);
            let block_name_len = u16::from_le_bytes(
                *raw.get(offset..=offset + 1)
                    .and_then(|bytes| bytes.as_array())
                    .ok_or(FileLoadError::InvalidImage)?,
            ) as usize;
            println!("Block name len: {}", block_name_len);
            offset += 2;
            let block_name = String::from_utf8_lossy(&raw[offset..offset + block_name_len]);
            println!("Block name:     {}", block_name);
            offset += block_name_len;
            let block_base = u64::from_le_bytes(
                *raw.get(offset..=offset+7)
                    .and_then(|bytes| bytes.as_array())
                    .ok_or(FileLoadError::InvalidImage)?,
            );
            println!("Block base:     0x{:08X}", block_base);
            offset += 8;
            let block_data_len = u32::from_le_bytes(
                *raw.get(offset..=offset + 3)
                    .and_then(|bytes| bytes.as_array())
                    .ok_or(FileLoadError::InvalidImage)?,
            ) as usize;
            println!("Block data len: 0x{:08X}", block_data_len);
            offset += 4;
            let block_data_raw = raw[offset..offset + block_data_len].to_vec();
            println!("Block data:\n{:?}", block_data_raw);
            offset += block_data_len;
            let mut mem = self.memory.write().map_err(|_|FileLoadError::MemLockPoisoned)?;
            mem[(block_base as usize)..(block_base as usize) + block_data_len]
                .copy_from_slice(&block_data_raw);
            blk_counter += 1;
            if block_name == "start" {
                start_block.0 = true;
                start_block.1 = block_base;
            }
            crate::core::runtime::logging::LOGGER
                .lock().map_err(|_| FileLoadError::LoggerLockPoisoned)?
                .info(&format!("Loaded block {}\n", block_name));
        }
        if start_block.0 {
            for core in &mut cpu.cores {
                core.pc = start_block.1;
            }
            Ok(())
        } else {
            Err(FileLoadError::NoStartBlock)
        }
    }
}

#[derive(Debug)]
pub enum FileLoadError {
    Io(std::io::Error),
    NoStartBlock,
    InvalidImage,
    MemLockPoisoned,
    LoggerLockPoisoned,
}

impl From<std::io::Error> for FileLoadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug)]
pub struct MemoryRegion {
    start: u64,
    end: u64,
    device: Box<dyn Device>,
}

pub trait Device {
    fn read8(&self, addr: u64) -> u8;
    fn read16(&self, addr: u64) -> u16 {
        u16::from_le_bytes([self.read8(addr), self.read8(addr + 1)])
    }
    fn read32(&self, addr: u64) -> u32 {
        u32::from_le_bytes([
            self.read8(addr),
            self.read8(addr + 1),
            self.read8(addr + 2),
            self.read8(addr + 3),
        ])
    }
    fn write8(&self, addr: u64, value: u8);
    fn write16(&self, addr: u64, value: u16) {
        let bytes = value.to_le_bytes();
        self.write8(addr, bytes[0]);
        self.write8(addr + 1, bytes[1]);
    }
    fn write32(&self, addr: u64, value: u32) {
        let bytes = value.to_le_bytes();
        self.write8(addr, bytes[0]);
        self.write8(addr + 1, bytes[1]);
        self.write8(addr + 2, bytes[2]);
        self.write8(addr + 3, bytes[3]);
    }
}

impl std::fmt::Debug for dyn Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
