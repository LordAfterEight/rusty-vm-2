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

            memory.data[0x0] = 0x88;

            memory.data[0x8B] = (crate::OpCode::LOAD_IMM as u8) << 1;
            memory.data[0x8A] = 0b00010000;
            memory.data[0x89] = 0b00000000;
            memory.data[0x88] = 0b11111111;

            memory.data[0x8F] = (crate::OpCode::LOAD_IMM as u8) << 1;
            memory.data[0x8E] = 0b00100000;
            memory.data[0x8D] = 0b00010000;
            memory.data[0x8C] = 0b00000010;

            memory.data[0x93] = (crate::OpCode::STOR_BYTE as u8) << 1;
            memory.data[0x92] = 0b00100000;
            memory.data[0x91] = 0b10000000;
            memory.data[0x90] = 0b00000000;
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
