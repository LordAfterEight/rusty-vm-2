use std::io::Read;

#[derive(Debug)]
pub struct Memory {
    pub data: memmap2::MmapMut,
}

impl Memory {
    pub fn empty(size: usize) -> Self {
        info!("Allocating {} bytes of empty VM address space to system RAM...", size);
        let memory = memmap2::MmapOptions::new().len(size).map_anon().unwrap();
        Self {
            data: memory,
        }
    }
    pub fn from_file(path: &str, size: usize) -> Self {
        info!("Allocating {} bytes of VM address space to system RAM...", size);
        let mut memory = memmap2::MmapOptions::new().len(size).map_anon().unwrap();
        info!("Loading ROM...");
        let rom_data = Memory::get_data_from_file(path);
        memory[0..rom_data.len()].copy_from_slice(rom_data.as_slice());
        Self {
            data: memory,
        }
    }
    pub fn get_data_from_file(path: &str) -> Box<[u8; 0x1_0000_0000]> {
        let mut buf = vec![0u8];
        let mut file = std::fs::File::open(&path)
            .expect("Could not open File");
        file.read(buf.as_mut_slice()).unwrap();
        let mut rom = Box::new([0u8;0x1_0000_0000]);
        for (i, &byte) in buf.iter().enumerate() {
            rom[i] = byte;
        }
        return rom
    }
}

impl crate::mmio::AddressSpace for Memory {
    fn read(&self, addr: u32) -> u8 {
        self.data[addr as usize]
    }
    fn write(&mut self, addr: u32, value: u8) {
        self.data[addr as usize] = value;
    }
}
