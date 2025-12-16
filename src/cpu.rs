use std::io::{Read};

use crate::opcodes::{OpCode};

/// A 32-bit CPU with indirect register addressing
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
pub struct CPU<'a> {
    pub mode: CpuMode,
    pub memory: &'a mut crate::memory::Memory,
    pub program_counter: usize,
    pub stack_pointer: u32,
    pub instruction: u32,
    pub registers: [u32; 32],
}

impl<'a> CPU<'a> {
    pub fn new(mode: CpuMode, memory: &'a mut crate::memory::Memory) -> Self {
        Self {
            mode,
            memory,
            program_counter: 0x0000_0000,
            stack_pointer: 0x4000_0000,
            instruction: 0b00000000_00000000_00000000_00000000,
            registers: [0; 32],
        }
    }

    /// Advances the program counter by one. Wrapping
    fn advance_pc(&mut self) {
        if self.program_counter < 0x4000_0000 {
            self.program_counter += 1;
        } else {
            self.program_counter = 0;
        }
    }

    /// Advances the stack pointer by one. Wrapping
    fn advance_sp(&mut self) {
        if self.stack_pointer < 0x8000_0000 {
            self.stack_pointer += 1;
        } else {
            self.stack_pointer = 0x4000_0000;
        }
    }

    /// Moves the stack pointer back by one. Wrapping
    fn decrease_sp(&mut self) {
        if self.stack_pointer > 0x4000_0000 {
            self.stack_pointer -= 1;
        } else {
            self.stack_pointer = 0x7FFF_FFFF;
        }
    }

    fn write_u32_to_ram(&mut self, value: u32) {
        let value = value.to_le_bytes();
        for i in 0..4 {
            self.memory.data[self.stack_pointer as usize] = value[i];
            self.advance_sp();
        }
        println!("Stored {:032b} to RAM at addresses 0x{:08X} - 0x{:08X}", u32::from_le_bytes(value), self.stack_pointer, self.stack_pointer + 4);
    }

    fn read_u32_from_ram(&mut self) -> u32 {
        let mut value: [u8; 4] = [0; 4];
        for i in 0..4 {
            self.decrease_sp();
            value[i] = self.memory.data[self.stack_pointer as usize];
        }
        println!("Read u32 {:032b} from RAM at addresses 0x{:08X} - 0x{:08X}", u32::from_be_bytes(value), self.stack_pointer, self.stack_pointer + 4);
        return u32::from_be_bytes(value);
    }

    fn pop_u32_from_ram(&mut self) -> u32 {
        let mut value: [u8; 4] = [0; 4];
        for i in 0..4 {
            self.decrease_sp();
            value[i] = self.memory.data[self.stack_pointer as usize];
            self.memory.data[self.stack_pointer as usize] = 0;
        }
        println!("Read u32 {:032b} from RAM at addresses 0x{:08X} - 0x{:08X}", u32::from_be_bytes(value), self.stack_pointer, self.stack_pointer + 4);
        return u32::from_be_bytes(value);
    }

    fn get_instruction(&mut self) {
        let mut instruction: [u8; 4] = [0; 4];
        for i in 0..4 {
            instruction[i] = self.memory.data[self.program_counter];
            self.advance_pc();
        }
        self.instruction = u32::from_le_bytes(instruction);
    }

    fn handle_errors(&self, error: CpuError) {
        let severity = match error.is_severe() {
            true => "Severe",
            false => "Minor"
        };
        match self.mode {
            CpuMode::Safe => {
                println!("{} {}", severity, error);
                println!("Shutting down VM...");
                std::process::exit(1);
            }
            CpuMode::Stable => {
                println!("{} {}", severity, error);
                if error.is_severe() {
                    println!("Shutting down VM...");
                    std::process::exit(1);
                } else {
                    println!("Ignoring error...\n");
                }
            }
            CpuMode::Unstable => {
                println!("{} {}", severity, error);
                println!("Ignoring error...\n");
            }
            CpuMode::Debug => {
                println!("{} {}", severity, error);
                println!("Program Counter: 0x{:08X}", self.program_counter);
                println!("Stack Pointer: 0x{:08X}", self.stack_pointer);
                println!("Registers:\n{:?}", self.registers);
                loop {
                    let mut input = [0u8; 1];
                    std::io::stdin().read_exact(&mut input).unwrap();
                    if input[0] == b'\n' { break }
                }
            }
        }
    }

    fn tick(&mut self) -> Result<(), CpuError> {
        self.get_instruction();
        let opcode_val = (self.instruction >> 25) & 0x7F;
        let opcode = match TryFrom::try_from(opcode_val) {
            Ok(opcode) => opcode,
            Err(_) => {
                println!("Failed to decode OpCode at 0x{:08X}", self.program_counter - 4);
                OpCode::NOOP
            }
        };
        println!("\n0x{:08X}: 0x{:02X} - {}", self.program_counter - 4, opcode_val, opcode);
        match opcode {
            OpCode::LOAD_IMM => {
                let register = (self.instruction >> 20) & 0x1F;
                let value = self.instruction & 0xFFFFF;
                self.registers[register as usize] = value;
                println!("Loaded value {} into register {}", value, register);
            },
            OpCode::JUMP_IMM => {
                let addr = self.instruction & 0x1FFFFFF;
                self.program_counter = addr as usize;
            },
            OpCode::JUMP_REG => {
                let register = self.instruction & 0x1F;
                self.program_counter = self.registers[register as usize] as usize;
            },
            OpCode::BRAN_IMM => {
                self.write_u32_to_ram(self.program_counter as u32);
                let addr = self.instruction & 0x1FFFFFF;
                self.program_counter = addr as usize;
            },
            OpCode::BRAN_REG => {
                self.write_u32_to_ram(self.program_counter as u32);
                let register = self.instruction & 0x1F;
                self.program_counter = self.registers[register as usize] as usize;
            },
            OpCode::RTRN => {
                let addr = self.read_u32_from_ram();
                self.program_counter = addr as usize;
            }
            OpCode::RTRN_POP => {
                let addr = self.pop_u32_from_ram();
                self.program_counter = addr as usize;
            }
            OpCode::DIV => {
            },
            OpCode::NOOP => {
            }
            _ => return Err(CpuError::new(self.program_counter, CpuErrorType::UnimplementedOpCode(opcode)))
        }
        Ok(())
    }

    pub fn update(&mut self) {
        match self.tick() {
            Ok(_) => {}
            Err(e) => self.handle_errors(e),
        }
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
#[display("CPU error occured at {:#010X}: {error_type}", program_counter - 4)]
pub struct CpuError {
    #[deref]
    pub error_type: CpuErrorType,
    pub program_counter: usize
}

impl CpuError {
    pub fn new(program_counter: usize, error_type: CpuErrorType) -> Self {
        Self {
            error_type,
            program_counter
        }
    }
}

#[derive(Debug, Display)]
pub enum CpuErrorType {
    StackOverflow,
    #[display("Invalid instruction: {:#010X}", _0)]
    InvalidInstruction(u32),
    #[display("Unimplemented OpCode: {:#?}", _0)]
    UnimplementedOpCode(OpCode),
    Halt,
    DivisionByZero,
    StackOpOutOfBounds,
}

impl Severity for CpuErrorType {
    fn is_minor(&self) -> bool {
        match self {
            CpuErrorType::StackOverflow => true,
            CpuErrorType::InvalidInstruction(_) => true,
            CpuErrorType::UnimplementedOpCode(_) => true,
            CpuErrorType::Halt => true,
            CpuErrorType::DivisionByZero => false,
            CpuErrorType::StackOpOutOfBounds => false,
        }
    }
}

pub trait Severity {
    fn is_minor(&self) -> bool;
    fn is_severe(&self) -> bool {
        !self.is_minor()
    }
}
