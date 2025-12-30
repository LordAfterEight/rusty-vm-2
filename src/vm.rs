pub struct VM {
    pub cpu: crate::cpu::CPU,
}

impl VM {
    pub fn new() -> Self {
        let mut memory = crate::memory::Memory::empty(0x1_0000_0000);

        /*
        memory.data[0x0] = 0x18; // Core 0 reset addr
        memory.data[0x4] = 0x84; // Core 1 reset addr
        memory.data[0x27] = (OpCode::IRPT_SEND as u8) << 1;
        // xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx
        memory.data[0x26] = 0b00010000;
        memory.data[0x25] = 0b10000000;

        memory.data[0x87] = (OpCode::IRPT_SEND as u8) << 1;
        memory.data[0x86] = 0b00000001;
        memory.data[0x85] = 0b00000000;

        */

        memory.data[0x0] = 0x88;

        memory.data[0x8B] = (crate::OpCode::LDUP_IMM as u8) << 1;
        memory.data[0x8A] = 0b00011111;
        memory.data[0x89] = 0b11111111;
        memory.data[0x88] = 0b11111111;

        memory.data[0x8F] = (crate::OpCode::ORI as u8) << 1;
        memory.data[0x8E] = 0b00011000;
        memory.data[0x8D] = 0b00001111;
        memory.data[0x8C] = 0b11111111;

        memory.data[0x93] = (crate::OpCode::LDUP_IMM as u8) << 1;
        memory.data[0x92] = 0b00101111;
        memory.data[0x91] = 0b11111111;
        memory.data[0x90] = 0b11111111;

        memory.data[0x97] = (crate::OpCode::ORI as u8) << 1;
        memory.data[0x96] = 0b00101000;
        memory.data[0x95] = 0b00001111;
        memory.data[0x94] = 0b11111111;

        memory.data[0x9B] = ((crate::OpCode::BRAN_REL as u8) << 1) | 0b0;
        memory.data[0x9A] = 0b10000000;

        let mem = std::sync::Arc::new(std::sync::RwLock::new(memory));

        let cpu = crate::cpu::CPU::new(crate::cpu::CpuMode::Debug, std::sync::Arc::clone(&mem));
        Self {
            cpu,
        }
    }

    pub fn run(self) {
        let mut handles = Vec::new();
        info!("Starting VM in {} mode...", format!("{}", self.cpu.mode));

        let mut cpu = self.cpu;
        let cpu_handle = std::thread::Builder::new()
            .name("Rusty-VM-CPU".to_string())
            .spawn(move || {
                info!("Starting CPU...");
                cpu.run();
            })
            .unwrap();
        handles.push(cpu_handle);

        let gpu_handle = std::thread::Builder::new()
            .name("Rusty-VM-GPU".to_string())
            .spawn(move || {
                info!("Starting GPU...");
                crate::gpu::main();
            })
            .unwrap();
        handles.push(gpu_handle);

        for handle in handles {
            handle.join().unwrap();
        }

        loop {}
    }
}
