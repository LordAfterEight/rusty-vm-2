use crate::OpCode;
use crate::cpu::{Interrupt, InterruptType, CpuError, CpuErrorType};

#[derive(Debug)]
pub struct Core {
    pub program_counter: u32,
    pub stack_pointer: u32,
    pub registers: [u32; 32],
    pub eq_flag: bool,
    pub index: u32,
    pub busy: bool,
    pub halted: bool,
    pub receiver: std::sync::mpsc::Receiver<Interrupt>,
    pub senders: [std::sync::mpsc::Sender<Interrupt>; 4],
}

impl Core {
    pub fn new(
        index: u32,
        senders: [std::sync::mpsc::Sender<Interrupt>; 4],
        receiver: std::sync::mpsc::Receiver<Interrupt>,
        memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>
    ) -> Self {
        info!("Created Core with index {index}");
        let mut core = Self {
            program_counter: 0x0000_0000 + index * 4,
            stack_pointer: 0x4000_0000,
            registers: [0; 32],
            eq_flag: false,
            index: index,
            busy: false,
            halted: false,
            senders,
            receiver,
        };
        core.reset_hard(memory);
        return core
    }

    fn reset_soft(&mut self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>) {
        self.program_counter = 0x0 + self.index * 4;
        let new_addr = self.fetch_u32(memory);
        self.program_counter = new_addr;
        self.stack_pointer = 0x4000_0000;
    }

    fn reset_hard(&mut self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>) {
        self.reset_soft(memory);
        for register in self.registers.iter_mut() {
            *register = 0;
        }
    }

    /// Advances the program counter by one. Wrapping.
    fn advance_pc(&mut self) {
        if self.program_counter < 0x4000_0000 {
            self.program_counter += 1;
        } else {
            self.program_counter = 0;
        }
    }

    /// Advances the stack pointer by one. Wrapping.
    fn advance_sp(&mut self) {
        if self.stack_pointer < 0x8000_0000 {
            self.stack_pointer += 1;
        } else {
            self.stack_pointer = 0x4000_0000;
        }
    }

    /// Moves the stack pointer back by one. Wrapping.
    fn decrease_sp(&mut self) {
        if self.stack_pointer > 0x4000_0000 {
            self.stack_pointer -= 1;
        } else {
            self.stack_pointer = 0x7FFF_FFFF;
        }
    }

    fn write_byte(&self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>, address: u32, value: u8) {
        let mut mem = memory.lock().unwrap();
        mem.data[address as usize] = value;
    }

    fn read_byte(&self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>, address: u32) -> u8 {
        let mem = memory.lock().unwrap();
        mem.data[address as usize]
    }

    fn write_u32_to_ram(&mut self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>, value: u32) {
        let value = value.to_le_bytes();
        for i in 0..4 {
            self.write_byte(memory, self.stack_pointer, value[i]);
            self.advance_sp();
        }
        info!(
            "Stored {:032b} to RAM at addresses 0x{:08X} - 0x{:08X}",
            u32::from_le_bytes(value),
            self.stack_pointer,
            self.stack_pointer + 4
        );
    }

    fn read_u32_from_ram(&mut self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>) -> u32 {
        let mut value: [u8; 4] = [0; 4];
        for i in 0..4 {
            self.decrease_sp();
            value[i] = self.read_byte(memory, self.stack_pointer);
        }
        info!(
            "Read u32 {:032b} from RAM at addresses 0x{:08X} - 0x{:08X}",
            u32::from_be_bytes(value),
            self.stack_pointer,
            self.stack_pointer + 4
        );
        return u32::from_be_bytes(value);
    }

    fn pop_u32_from_ram(&mut self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>) -> u32 {
        let mut value: [u8; 4] = [0; 4];
        for i in 0..4 {
            self.decrease_sp();
            let mut mem = memory.lock().unwrap();
            value[i] = mem.data[self.stack_pointer as usize];
            mem.data[self.stack_pointer as usize] = 0;
        }
        info!(
            "Read u32 {:032b} from RAM at addresses 0x{:08X} - 0x{:08X}",
            u32::from_be_bytes(value),
            self.stack_pointer,
            self.stack_pointer + 4
        );
        return u32::from_be_bytes(value);
    }

    fn fetch_u32(&mut self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>) -> u32 {
        let mut instruction: [u8; 4] = [0; 4];
        let mem = memory.lock().unwrap();
        for i in 0..4 {
            instruction[i] = mem.data[self.program_counter as usize];
            self.advance_pc();
        }
        u32::from_le_bytes(instruction)
    }

    pub fn handle_interrupts(&mut self, interrupt: Interrupt, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>) {
        info!(core=self.index, "Core {} received {}", self.index, interrupt);
        match interrupt.interrupt_type {
            InterruptType::Halt => self.halted = false,
            InterruptType::Resume => self.halted = true,
            InterruptType::SoftReset => self.reset_soft(&memory),
            InterruptType::HardReset => self.reset_hard(&memory),
        }
    }

    pub fn tick(&mut self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>) -> Result<(), CpuError> {
        let instruction = self.fetch_u32(&memory);
        let opcode_val = (instruction >> 25) & 0x7F;
        let opcode = match TryFrom::try_from(opcode_val) {
            Ok(opcode) => opcode,
            Err(_) => {
                error!(
                    core = self.index,
                    "Failed to decode OpCode at 0x{:08X}",
                    self.program_counter - 4
                );
                OpCode::NOOP
            }
        };
        info!(
            core = self.index,
            "0x{:08X}: 0x{:02X} - {}",
            self.program_counter - 4,
            opcode_val,
            opcode
        );
        match opcode {
            OpCode::LOAD_IMM => {
                let rde = (instruction >> 20) & 0x1F;
                let value = instruction & 0xFFFFF;
                self.registers[rde as usize] = value;
                info!(core=?self.index, "Loaded value {} into register {}", value, rde);
            },
            OpCode::LDUP_IMM => {
                let rde = (instruction >> 20) & 0x1F;
                let value = instruction & 0xFFFFF;
                self.registers[rde as usize] = value << 12;
                info!(core=?self.index, "Loaded value {} into register {}", value, rde);
            },
            OpCode::LOAD_BYTE => {
                let rde = (instruction >> 20) & 0x1F;
                let addr = (instruction >> 15) & 0x1F;
                let value = self.read_byte(memory, addr);
                self.registers[rde as usize] = value as u32;
                info!(core=?self.index, "Read value {} from 0x{:08X}", value, addr);
            },
            OpCode::STOR_BYTE => {
                let addr = (instruction >> 20) & 0x1F;
                let value = self.registers[((instruction >> 15) & 0x1F) as usize];
                info!(core=?self.index, "Writing value {} to 0x{:08X}", value, addr);
                self.write_byte(memory, addr, value as u8);
            }
            OpCode::JUMP_IMM => {
                let addr = instruction & 0x1FFFFFF;
                info!(core=?self.index, "Jumping to address 0x{:08X}", addr);
                self.program_counter = addr;
            },
            OpCode::JUMP_REG => {
                let rs1 = instruction & 0x1F;
                info!(core=?self.index, "Jumping to address 0x{:08X}", self.registers[rs1 as usize]);
                self.program_counter = self.registers[rs1 as usize];
            },
            OpCode::BRAN_IMM => {
                self.write_u32_to_ram(&memory, self.program_counter as u32);
                let addr = instruction & 0x1FFFFFF;
                info!(core=?self.index, "Branching to address 0x{:08X}", addr);
                self.program_counter = addr;
            },
            OpCode::BRAN_REG => {
                self.write_u32_to_ram(&memory, self.program_counter as u32);
                let rs1 = instruction & 0x1F;
                info!(core=?self.index, "branching to address 0x{:08X}", self.registers[rs1 as usize]);
                self.program_counter = self.registers[rs1 as usize];
            },
            OpCode::RTRN => {
                let addr = self.read_u32_from_ram(&memory);
                info!(core=?self.index, "Returning to address 0x{:08X}", addr);
                self.program_counter = addr;
            },
            OpCode::RTRN_POP => {
                let addr = self.pop_u32_from_ram(&memory);
                info!(core=?self.index, "Returning to address 0x{:08X}", addr);
                self.program_counter = addr;
            },
            OpCode::ORR => {
                let rde = (instruction >> 20) & 0x1F;
                let rs1 = (instruction >> 15) & 0x1F;
                let rs2 = (instruction >> 10) & 0x1F;
                info!(core=?self.index, "OR-ing register {} and register {}, storing in register {}", rs1, rs2, rde);
                self.registers[rde as usize] = self.registers[rs1 as usize] | self.registers[rs2 as usize];
            },
            OpCode::ORI => {
                let rde = (instruction >> 20) & 0x1F;
                let value = instruction & 0xFFFFF;
                info!(core=?self.index, "OR-ing register {} with immediate value {}, storing in register {}", rde, value, rde);
                self.registers[rde as usize] = self.registers[rde as usize] | value;
            },
            OpCode::XOR => {
                let rde = (instruction >> 20) & 0x1F;
                let rs1 = (instruction >> 15) & 0x1F;
                let rs2 = (instruction >> 10) & 0x1F;
                info!(core=?self.index, "XOR-ing register {} and register {}, storing in register {}", rs1, rs2, rde);
                self.registers[rde as usize] = self.registers[rs1 as usize] ^ self.registers[rs2 as usize];
            },
            OpCode::AND => {
                let rde = (instruction >> 20) & 0x1F;
                let rs1 = (instruction >> 15) & 0x1F;
                let rs2 = (instruction >> 10) & 0x1F;
                info!(core=?self.index, "AND-ing register {} and register {}, storing in register {}", rs1, rs2, rde);
                self.registers[rde as usize] = self.registers[rs1 as usize] & self.registers[rs2 as usize];
            },
            OpCode::ADD => {
                let rde = (instruction >> 20) & 0x1F;
                let rs1 = (instruction >> 15) & 0x1F;
                let rs2 = (instruction >> 10) & 0x1F;
                info!(core=?self.index, "Adding register {} and register {}, storing in register {}", rs1, rs2, rde);
                let value = (self.registers[rs1 as usize] as u64) + (self.registers[rs2 as usize] as u64);
                if value > u32::MAX.into() {
                    self.registers[rde as usize] = (value >> 1) as u32;
                    return Err(CpuError::new(self.program_counter, self.stack_pointer, self.registers, CpuErrorType::AddWithOverflow, self.index))
                } else {
                    self.registers[rde as usize] = value as u32;
                }

            },
            OpCode::SUB => {
                let rde = (instruction >> 20) & 0x1F;
                let rs1 = (instruction >> 15) & 0x1F;
                let rs2 = (instruction >> 10) & 0x1F;
                info!(core=?self.index, "Subtracting register {} from register {}, storing in register {}", rs2, rs1, rde);
                if self.registers[rs1 as usize] >= self.registers[rs2 as usize] {
                    self.registers[rde as usize] = self.registers[rs1 as usize] - self.registers[rs2 as usize];
                } else {
                    return Err(CpuError::new(self.program_counter, self.stack_pointer, self.registers, CpuErrorType::SubWithOverflow, self.index))
                }
            },
            OpCode::NOOP => {
            },
            OpCode::RSET_SOFT => self.reset_soft(memory),
            OpCode::RSET_HARD => self.reset_hard(memory),
            OpCode::HALT => {
                return Err(CpuError::new(
                    self.program_counter,
                    self.stack_pointer,
                    self.registers,
                    CpuErrorType::Halt,
                    self.index,
                ));
            },
            OpCode::IRPT_SEND => {
                let target_idx = (instruction >> 20) & 0x1F;
                let itype_val = (instruction >> 15) & 0x1F;

                if let Some(target_sender) = self.senders.get(target_idx as usize) {
                    let msg = Interrupt {
                        sender_id: self.index,
                        interrupt_type: match itype_val {
                            1 => InterruptType::Resume,
                            2 => InterruptType::Halt,
                            3 => InterruptType::SoftReset,
                            4 => InterruptType::HardReset,
                            _ => panic!("Unknown Interrupt: {}", itype_val)
                        },
                    };
                    info!(core=?self.index, "Sent {} to Core {}", msg, target_idx);
                    let _ = target_sender.send(msg);
                }
            },
            _ => {
                return Err(CpuError::new(
                    self.program_counter,
                    self.stack_pointer,
                    self.registers,
                    CpuErrorType::UnimplementedOpCode(opcode),
                    self.index,
                ));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    }
}
