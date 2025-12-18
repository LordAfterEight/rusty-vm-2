use std::io::Read;

#[derive(Debug)]
pub struct Memory {
    pub data: Box<[u8]>,
}

impl Memory {
    pub fn empty() -> Self {
        println!("Allocating 4GiB of VM storage to system RAM...");
        let memory = vec![0u8; 0x1_0000_0000].into_boxed_slice();
        Self {
            data: memory,
        }
    }
    pub fn from_file(path: &str) -> Self {
        println!("Allocating 4GiB of VM storage to system RAM...");
        let mut memory = vec![0u8; 0x1_0000_0000].into_boxed_slice();
        println!("Loading ROM...");
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
