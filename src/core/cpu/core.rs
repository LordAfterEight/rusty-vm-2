use crate::core::{cpu::opcodes::OpCode, runtime::logging::LOGGER};

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
    pub sp: u64,
    pub registers: [crate::core::cpu::register::Register<'a, u32>; 32],
    pub flags: u8,
}

impl<'a> Core<'a> {
    pub fn new(name: &'a str) -> Self {
        let mut core = Self {
            name: name,
            bus: None,
            pc: 0,
            sp: 0xFFFFFFFB,
            registers: [crate::core::cpu::register::Register::<'a, u32>::create("GPR"); 32],
            flags: 0b0000_0000,
        };

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
            if self.flags >> 5 & 0x1 == 0 {
                let instr = crate::core::cpu::opcodes::Decoded::try_from(bus.read32(self.pc))
                    .map_err(CoreError::DecodingError)?;
                self.pc += 4;
                match instr.op {
                    OpCode::LDIV => {
                        self.registers[instr.rd].insert(instr.imm)?;
                        self.flags |= 0b10000000;
                    }
                    OpCode::LDRE => todo!("implement LDRE"),
                    OpCode::LD08 => {
                        self.registers[instr.rd]
                            .insert(bus.read8(self.registers[instr.rs1].extract() as u64))?;
                        self.flags |= 0b10000000;
                    }
                    OpCode::LD16 => {
                        self.registers[instr.rd]
                            .insert(bus.read16(self.registers[instr.rs1].extract() as u64))?;
                        self.flags |= 0b10000000;
                    }
                    OpCode::LD32 => {
                        self.registers[instr.rd]
                            .insert(bus.read32(self.registers[instr.rs1].extract() as u64))?;
                        self.flags |= 0b10000000;
                    }
                    OpCode::LDHW => todo!("implement LDHW"),
                    OpCode::LDWD => {
                        let val = bus.read32(self.pc);
                        self.pc += 4;
                        self.registers[instr.rd].insert(val)?;
                        self.flags |= 0b10000000;
                    }
                    OpCode::STIV => todo!("implement STIV"),
                    OpCode::STIA => todo!("implement STIA"),
                    OpCode::ST08 => {
                        bus.write8(
                            self.registers[instr.rs2].extract() as u64,
                            (self.registers[instr.rs1].extract() & 0xFF) as u8,
                        );
                    }
                    OpCode::ST16 => {
                        bus.write16(
                            self.registers[instr.rs2].extract() as u64,
                            (self.registers[instr.rs1].extract() & 0xFF) as u16,
                        );
                    },
                    OpCode::ST32 => {
                        bus.write32(
                            self.registers[instr.rs2].extract() as u64,
                            (self.registers[instr.rs1].extract() & 0xFF) as u32,
                        );
                    },
                    OpCode::JMIM => todo!("implement JMIM"),
                    OpCode::JMRE => todo!("implement JMRE"),
                    OpCode::JMRL => {
                        let raw = instr.imm & 0x01FF_FFFF;
                        let offset = if raw & 0x0100_0000 != 0 {
                            (raw as i32) - (1 << 25)
                        } else {
                            raw as i32
                        };
                        self.pc = self.pc.wrapping_add_signed(offset.into());
                    },
                    OpCode::JMIZ => {
                        if self.registers[instr.rs1].extract() == 0 {
                            let raw = instr.imm & 0x000F_FFFF;

                            let offset = if raw & 0x0008_0000 != 0 {
                                (raw as i32) - (1 << 20)
                            } else {
                                raw as i32
                            };

                            self.pc = self.pc.wrapping_add_signed(offset as i64);
                        }
                    }
                    OpCode::JMNZ => todo!("implement JMNZ"),
                    OpCode::BRIM => {
                        bus.write32(self.sp, self.pc as u32);
                        self.sp -= 4;
                        self.pc = instr.imm as u64;
                    }
                    OpCode::BRRE => todo!("implement BRRE"),
                    OpCode::BRRL => todo!("implement BRRL"),
                    OpCode::BRIZ => todo!("implement BRIZ"),
                    OpCode::BRNZ => todo!("implement BRNZ"),
                    OpCode::AADD => {
                        self.registers[instr.rd].insert(
                            self.registers[instr.rs1].extract() + self.registers[instr.rs2].extract(),
                        )?;
                    }
                    OpCode::ASUB => {
                        self.registers[instr.rd].insert(
                            self.registers[instr.rs1].extract() - self.registers[instr.rs2].extract(),
                        )?;
                    }
                    OpCode::AMUL => {
                        self.registers[instr.rd].insert(
                            self.registers[instr.rs1].extract() - self.registers[instr.rs2].extract(),
                        )?;
                    }
                    OpCode::LAND => {
                        self.registers[instr.rd].insert(
                            self.registers[instr.rs1].extract() & self.registers[instr.rs2].extract(),
                        )?;
                    }
                    OpCode::LORI => {
                        self.registers[instr.rd]
                            .insert(self.registers[instr.rd].extract() | instr.imm)?;
                    }
                    OpCode::LORR => {
                        self.registers[instr.rd].insert(
                            self.registers[instr.rs1].extract() | self.registers[instr.rs2].extract(),
                        )?;
                    }
                    OpCode::LXOR => {
                        self.registers[instr.rd].insert(
                            self.registers[instr.rs1].extract() ^ self.registers[instr.rs2].extract(),
                        )?;
                    }
                    OpCode::LROR => todo!("implement LROR"),
                    OpCode::LROL => todo!("implement LROL"),
                    OpCode::HALT => {
                        LOGGER.lock().unwrap().info("Core halted");
                        self.flags |= 0b00100000;
                    },
                    OpCode::PUSH => {
                        bus.write32(self.sp, self.registers[instr.rs1].extract());
                        self.sp -= 4;
                    }
                    OpCode::PULL => {
                        self.sp += 4;
                        self.registers[instr.rd].insert(bus.read32(self.sp))?;
                    }
                    OpCode::IRPT => todo!("implement IRPT"),
                    OpCode::SRES => todo!("implement SRES"),
                    OpCode::HRES => todo!("implement HRES"),
                    OpCode::RTRN => {
                        self.sp += 4;
                        self.pc = bus.read32(self.sp as u64) as u64;
                    },
                    OpCode::RTIZ => todo!("implement RTIZ"),
                    OpCode::RTNZ => todo!("implement RTNZ"),
                }
            }
            Ok(())
        } else {
            Err(CoreError::BusNotConnected)
        }
    }
}

#[derive(Debug)]
pub enum CoreError {
    DecodingError(crate::core::cpu::opcodes::DecodingError),
    RegisterError(crate::core::cpu::register::RegisterError),
    BusNotConnected,
}

impl From<crate::core::cpu::register::RegisterError> for CoreError {
    fn from(value: crate::core::cpu::register::RegisterError) -> Self {
        Self::RegisterError(value)
    }
}
