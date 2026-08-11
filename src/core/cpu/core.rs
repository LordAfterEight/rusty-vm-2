/// A 32-bit CPU Core
///
/// ## Flags
/// CPU flags are stored in a single 8-bit unsigned integer:
/// Zero | Carr | Halt | ---- | ---- | ---- | ----| ----
#[derive(Clone, Debug)]
pub struct Core<'a> {
    pub name: &'a str,
    pub bus: Option<std::sync::Arc<crate::core::runtime::bus::Bus>>,
    pub pc: u64,
    pub registers: [crate::core::cpu::register::Register<'a, u32>; 32],
    pub flags: u8,
}

impl<'a> Core<'a> {
    pub fn new(name: &'a str) -> Self {
        let mut core = Self {
            name: name,
            bus: None,
            pc: 0,
            registers: [crate::core::cpu::register::Register::<'a, u32>::create("GPR"); 32],
            flags: 0b0000_0000,
        };

        core.registers[0].name = "IPR";
        core.registers[1].name = "SPR";

        for i in 0..5 {
            core.registers[(32 - 5) + i].name = "ARG";
        }

        core
    }

    pub fn flag_zero(&self) -> bool {
        if self.flags >> 7 & 1 == 1 {
            return true;
        }
        false
    }

    pub fn flag_carr(&self) -> bool {
        if self.flags >> 6 & 1 == 1 {
            return true;
        }
        false
    }

    pub fn flag_halt(&self) -> bool {
        if self.flags >> 5 & 1 == 1 {
            return true;
        }
        false
    }

    pub fn tick(&mut self) -> Result<(), CoreError> {
        if let Some(bus) = &self.bus {
            let instr = crate::core::cpu::opcodes::Decoded::try_from(bus.read32(self.pc))
                .map_err(CoreError::DecodingError)?;
            println!("{:?}", instr);
            self.pc += 4;
            std::thread::sleep(std::time::Duration::from_millis(100));
            Ok(())
        } else {
            Err(CoreError::BusNotConnected)
        }
    }
}

#[derive(Debug)]
pub enum CoreError {
    DecodingError(crate::core::cpu::opcodes::DecodingError),
    BusNotConnected,
}
