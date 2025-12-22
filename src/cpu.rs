use std::io::Read;

use crate::opcodes::OpCode;

/// A 32-bit 4-Core CPU
///
/// # ==== General ====
///
/// - 32 General Purpose Registers
///
/// # ==== Instructions ====
/// Instructions are 32-bit words read from memory in 4x8-bit steps. An instruction can be
/// structured as follows:
///
/// `xxxxxxx xxxxx xxxxx xxxxx`
///  OpCode   rd1   rs1   rs2
#[derive(Debug)]
pub struct CPU {
    pub mode: CpuMode,
    pub memory: std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>,
    pub cores: [Option<Core>; 4],
    pub channel: (
        std::sync::mpsc::Sender<CpuError>,
        std::sync::mpsc::Receiver<CpuError>,
    ),
}

impl CPU {
    pub fn new(
        mode: CpuMode,
        memory: std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>,
    ) -> Self {
        let mut tx_rx_pairs: Vec<_> = (0..4).map(|_| std::sync::mpsc::channel()).collect();

        let all_senders: [std::sync::mpsc::Sender<Interrupt>; 4] = [
            tx_rx_pairs[0].0.clone(),
            tx_rx_pairs[1].0.clone(),
            tx_rx_pairs[2].0.clone(),
            tx_rx_pairs[3].0.clone(),
        ];

        let cores = std::array::from_fn(|i| {
            let (_own_tx, own_rx) = tx_rx_pairs.remove(0);
            let mut core = Core::new(i as u32, all_senders.clone(), own_rx, &memory);
            if i == 0 { core.busy = true; info!("Assigned busy to core {}", i)}
            Some(core)
        });

        Self {
            mode,
            memory,
            cores,
            channel: std::sync::mpsc::channel::<CpuError>(),
        }
    }

    fn handle_errors(&mut self, error: CpuError) {
        let severity = error.severity();
        info!(?severity, "Handling error: {}", error);
        match self.mode {
            CpuMode::Safe => {
                info!("Shutting down VM...");
                std::process::exit(1);
            }
            CpuMode::Stable => {
                if matches!(severity, CpuErrorSeverity::Severe) {
                    info!("Shutting down VM...");
                    std::process::exit(1);
                } else {
                    info!("Ignoring error...");
                }
            }
            CpuMode::Unstable => {
                info!("Ignoring error...");
            }
            CpuMode::Debug => {
                info!(
                    core=?error.core_index,
                    "\nProgram Counter: 0x{:08X}\nStack Pointer: 0x{:08X}\nRegisters: {:?}\n",
                    error.program_counter,
                    error.stack_pointer,
                    error.register_snapshot
                );
                info!(core=?error.core_index, "Press ENTER to let this core continue running");
                loop {
                    let mut input = [0u8; 1];
                    std::io::stdin().read_exact(&mut input).unwrap();
                    if input[0] == b'\n' {
                        break;
                    }
                }
            }
        }
    }

    pub fn run(&mut self) {
        let mut handles = Vec::new();

        for core in self.cores.iter_mut() {
            let mut core = core.take().unwrap();
            let memory = std::sync::Arc::clone(&self.memory);
            let tx = self.channel.0.clone();

            let handle = std::thread::Builder::new()
                .name(format!("RustyVM-Core-{}", core.index))
                .spawn(move || {
                    info!("Spawned thread: {}", std::thread::current().name().unwrap());
                    loop {
                        if let Ok(interrupt) = core.receiver.try_recv() {
                            core.handle_interrupts(interrupt, &memory);
                        }

                        if !core.busy {
                            if let Ok(interrupt) = core.receiver.recv() {
                                core.handle_interrupts(interrupt, &memory);
                            }
                            continue;
                        }

                        let result = {
                            core.tick(&memory)
                        };

                        if let Err(e) = result {
                            error!(core=core.index, "Core {} error: {}", core.index, e);
                            let error_severity = e.severity();
                            tx.send(e).unwrap();
                            if error_severity == CpuErrorSeverity::Minor {
                                loop {
                                    let mut input = [0u8; 1];
                                    std::io::stdin().read_exact(&mut input).unwrap();
                                    if input[0] == b'\n' {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                })
                .unwrap();

            handles.push(handle);
        }

        loop {
            match self.channel.1.recv() {
                Ok(error) => {
                    self.handle_errors(error);
                }
                Err(_) => break,
            }
        }
    }
}

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

impl<'a> Core {
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

    fn handle_interrupts(&mut self, interrupt: Interrupt, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>) {
        info!(core=self.index, "Core {} received {}", self.index, interrupt);
        match interrupt.interrupt_type {
            InterruptType::Halt => self.halted = false,
            InterruptType::Resume => self.halted = true,
            InterruptType::SoftReset => self.reset_soft(&memory),
            InterruptType::HardReset => self.reset_hard(&memory),
        }
    }

    fn tick(&mut self, memory: &std::sync::Arc<std::sync::Mutex<crate::memory::Memory>>) -> Result<(), CpuError> {
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
                info!("OR-ing register {} and register {}, storing in register {}", rs1, rs2, rde);
                self.registers[rde as usize] = self.registers[rs1 as usize] | self.registers[rs2 as usize];
            },
            OpCode::ORI => {
                let rde = (instruction >> 20) & 0x1F;
                let value = instruction & 0xFFFFF;
                info!("OR-ing register {} with immediate value {}, storing in register {}", rde, value, rde);
                self.registers[rde as usize] = self.registers[rde as usize] | value;
            },
            OpCode::XOR => {
                let rde = (instruction >> 20) & 0x1F;
                let rs1 = (instruction >> 15) & 0x1F;
                let rs2 = (instruction >> 10) & 0x1F;
                info!("XOR-ing register {} and register {}, storing in register {}", rs1, rs2, rde);
                info!("{:032b} ^ {:032b} => {:032b}", self.registers[rs1 as usize], self.registers[rs2 as usize], self.registers[rs1 as usize] ^ self.registers[rs2 as usize]);
                self.registers[rde as usize] = self.registers[rs1 as usize] ^ self.registers[rs2 as usize];
            },
            OpCode::AND => {
                let rde = (instruction >> 20) & 0x1F;
                let rs1 = (instruction >> 15) & 0x1F;
                let rs2 = (instruction >> 10) & 0x1F;
                info!("AND-ing register {} and register {}, storing in register {}", rs1, rs2, rde);
                self.registers[rde as usize] = self.registers[rs1 as usize] & self.registers[rs2 as usize];
            },
            OpCode::ADD => {
                let rde = (instruction >> 20) & 0x1F;
                let rs1 = (instruction >> 15) & 0x1F;
                let rs2 = (instruction >> 10) & 0x1F;
                info!("Adding register {} and register {}, storing in register {}", rs1, rs2, rde);
                let value = (self.registers[rs1 as usize] as u64) + (self.registers[rs2 as usize] as u64);
                if value > u32::MAX.into() {
                    self.registers[rde as usize] = (value >> 1) as u32;
                    return Err(CpuError::new(self.program_counter, self.stack_pointer, self.registers, CpuErrorType::AddWithOverflow, self.index))
                } else {
                    self.registers[rde as usize] = value as u32;
                }

            }
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
                    info!("Core {} sent {} to Core {}", self.index, msg, target_idx);
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

#[derive(Debug, Display)]
/// Determines how the VM handles runtime Errors
pub enum CpuMode {
    /// Makes the VM crash gracefully when any runtime error occurs.
    Safe,
    /// Ignores minor runtime errors.
    Stable,
    /// Ignores all errors. Use at your own risk as ROM corruption may occur.
    Unstable,
    /// Dumps CPU and RAM data to the current directory on any runtime error and halts the VM.
    Debug,
}

#[derive(Debug, Display, Error, Deref)]
#[display("{} {}: {}", format!("CPU error occured in Core:{} at", core_index), format!("0x{:08X}", program_counter - 4), error_type)]
pub struct CpuError {
    #[deref]
    pub error_type: CpuErrorType,
    pub program_counter: u32,
    pub stack_pointer: u32,
    pub register_snapshot: [u32; 32],
    pub core_index: u32,
}

impl CpuError {
    pub fn new(program_counter: u32, stack_pointer: u32, register_snapshot: [u32; 32], error_type: CpuErrorType, core_index: u32) -> Self {
        Self {
            error_type,
            program_counter,
            stack_pointer,
            register_snapshot,
            core_index,
        }
    }
}

#[derive(PartialEq, Debug, Display)]
pub enum CpuErrorSeverity {
    Severe,
    Minor,
}

#[derive(Debug, Display, PartialEq)]
pub enum CpuErrorType {
    StackOverflow,
    #[display("Invalid instruction: {:#010X}", _0)]
    InvalidInstruction(u32),
    #[display("Unimplemented OpCode: {:#?}", _0)]
    UnimplementedOpCode(OpCode),
    Halt,
    DivisionByZero,
    StackOpOutOfBounds,
    AddWithOverflow,
}

pub trait Severity {
    fn severity(&self) -> CpuErrorSeverity;
}

impl Severity for CpuErrorType {
    fn severity(&self) -> CpuErrorSeverity {
        match self {
            CpuErrorType::StackOverflow => CpuErrorSeverity::Severe,
            CpuErrorType::InvalidInstruction(_) => CpuErrorSeverity::Severe,
            CpuErrorType::UnimplementedOpCode(_) => CpuErrorSeverity::Severe,
            CpuErrorType::Halt => CpuErrorSeverity::Severe,
            CpuErrorType::DivisionByZero => CpuErrorSeverity::Minor,
            CpuErrorType::StackOpOutOfBounds => CpuErrorSeverity::Minor,
            CpuErrorType::AddWithOverflow => CpuErrorSeverity::Minor,
        }
    }
}

#[derive(Debug, Display)]
#[display("Interrupt {:?}", interrupt_type)]
pub struct Interrupt {
    sender_id: u32,
    interrupt_type: InterruptType,
}

#[derive(Debug, Display)]
pub enum InterruptType {
    Resume,
    Halt,
    SoftReset,
    HardReset,
}
