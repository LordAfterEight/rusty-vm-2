pub struct VM {
    pub cpu: crate::cpu::CPU,
    pub bus: std::sync::Arc<std::sync::RwLock<crate::mmio::Bus>>,
    pub running: std::sync::Arc<std::sync::atomic::AtomicBool>
}

impl VM {
    pub fn new() -> Self {
        let bus = crate::mmio::Bus::new_empty(0x1_0000_0000);
        {
            let mut memory = bus.ram.write().unwrap();

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

            memory.data[0x0] = 0x20;

            // Load update enable value into r3 (Can be any value above 0)
            memory.data[0x23] = (crate::OpCode::LOAD_IMM as u8) << 1;
            memory.data[0x22] = 0b00110000; // r3
            memory.data[0x21] = 0b00000000;
            memory.data[0x20] = 0b00000001; // 255

            // Load GPU update enable register address into r2
            memory.data[0x27] = (crate::OpCode::LOAD_IMM as u8) << 1;
            memory.data[0x26] = 0b00100000; // r2
            memory.data[0x25] = 0b00010000; // |
            memory.data[0x24] = 0b00000010; // --> GPU register 2 at 0x4098

            // Store update enable value to update enable register of GPU
            memory.data[0x2B] = (crate::OpCode::STOR_BYTE as u8) << 1;
            memory.data[0x2A] = 0b00100001; // store to address in r2
            memory.data[0x29] = 0b10000000; // value from r3
            memory.data[0x28] = 0b00000000;

            // Load pixel color into r1
            memory.data[0x2F] = (crate::OpCode::LOAD_IMM as u8) << 1;
            memory.data[0x2E] = 0b00010000; // r1
            memory.data[0x2D] = 0b00000000; // |
            memory.data[0x2C] = 0b00000000; // --> Some color


            // Load frame buffer pointer to r0
            memory.data[0x33] = (crate::OpCode::LOAD_IMM as u8) << 1;
            memory.data[0x32] = 0b00000000; // r0 (fb pointer)
            memory.data[0x31] = 0b00000000;
            memory.data[0x30] = 0b00000000; // 0

            // Load incrementer into r4
            memory.data[0x37] = (crate::OpCode::LOAD_IMM as u8) << 1;
            memory.data[0x36] = 0b01000000; // r4 (incrementer)
            memory.data[0x35] = 0b00000000; //
            memory.data[0x34] = 0b00000001; // 1

            // Load GPU frame buffer register address into r5
            memory.data[0x3B] = (crate::OpCode::LOAD_IMM as u8) << 1;
            memory.data[0x3A] = 0b01010000; // r5 (fb address)
            memory.data[0x39] = 0b00010000; // |
            memory.data[0x38] = 0b00000000; // --> GPU register 0 at 0x4096

            // Store new frame buffer pointer into fb register of GPU
            memory.data[0x3F] = (crate::OpCode::STOR_BYTE as u8) << 1;
            memory.data[0x3E] = 0b01010000; // store to address in r5
            memory.data[0x3D] = 0b00000000; // value from r0
            memory.data[0x3C] = 0b00000000; //

            // Load GPU pixeldata register address into r6
            memory.data[0x43] = (crate::OpCode::LOAD_IMM as u8) << 1;
            memory.data[0x42] = 0b01100000; // r6 (pixeldata address)
            memory.data[0x41] = 0b00010000; // |
            memory.data[0x40] = 0b00000001; // --> GPU register 1 at 0x4097


            // Store pixeldata to GPU pixeldata register
            memory.data[0x47] = (crate::OpCode::STOR_BYTE as u8) << 1;
            memory.data[0x46] = 0b01100000; // store to address in r6
            memory.data[0x45] = 0b10000000; // value from r1
            memory.data[0x44] = 0b00000000; //

            // Increment frame buffer pointer to then be sent to GPU
            memory.data[0x4B] = (crate::OpCode::ADD as u8) << 1;
            memory.data[0x4A] = 0b00000010; // r0 (fb pointer)
            memory.data[0x49] = 0b00000000;
            memory.data[0x48] = 0b00000000;

            // Store new frame buffer pointer into fb register of GPU
            memory.data[0x4F] = (crate::OpCode::STOR_BYTE as u8) << 1;
            memory.data[0x4E] = 0b01010000; // store to address in r5
            memory.data[0x4D] = 0b00000000; // value from r0
            memory.data[0x4C] = 0b00000000; //

            // Repeat from address 0x48
            memory.data[0x53] = (crate::OpCode::JUMP_REL as u8) << 1;
            memory.data[0x52] = 0b00000000;
            memory.data[0x51] = 0b00000000;
            memory.data[0x50] = 0b00001100;
        }

        let bus = std::sync::Arc::new(std::sync::RwLock::new(bus.clone()));

        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let cpu = crate::cpu::CPU::new(crate::cpu::CpuMode::Debug, bus.clone(), running.clone());
        Self {
            cpu,
            bus,
            running
        }
    }

    pub fn run(self) {
        let mut handles = Vec::new();
        let running = self.running.clone();
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

        let gpu = std::sync::Arc::new(std::sync::Mutex::new(crate::gpu::GPU::init(0x1000)));
        self.bus.write().unwrap().regions.push(crate::mmio::MmioRegion {
            name: "GPU".to_string(),
            base: 0x1000,
            size: 0x10,
            device: gpu.clone()
        });
        let gpu_handle = std::thread::Builder::new()
            .name("Rusty-VM-GPU".to_string())
            .spawn(move || {
                info!("Starting GPU...");
                let mut window = minifb::Window::new(
                    "RustyVM - 2",
                    (1280) as usize,
                    (720) as usize,
                    minifb::WindowOptions {
                        resize: false,
                        scale: minifb::Scale::X1,
                        scale_mode: minifb::ScaleMode::Stretch,
                        ..Default::default()
                    }
                ).unwrap();
                window.set_target_fps(60);
                window.set_cursor_visibility(false);
                while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
                    let fb = {
                        let gpu_guard = gpu.lock().unwrap();
                        gpu_guard.frame_buffer.clone()
                    };
                    {
                        gpu.lock().unwrap().update().unwrap();
                    }
                    window.update_with_buffer(fb.as_slice() , 1280, 720)
                        .unwrap();
                }
                running.store(false, std::sync::atomic::Ordering::Relaxed);
                info!("Terminating threads...")
            })
            .unwrap();
        handles.push(gpu_handle);

        for handle in handles {
            handle.join().unwrap();
        }

        loop {}
    }
}
