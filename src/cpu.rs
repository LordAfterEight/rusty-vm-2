use std::io::{Read, Write};

use crate::opcodes::{self, OpCode};

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
    pub memory: &'a crate::memory::Memory,
    pub program_counter: usize,
    pub stack_pointer: u32,
    pub instruction: u32,
    pub registers: [u32; 32],
}

impl<'a> CPU<'a> {
    pub fn new(mode: CpuMode, memory: &'a crate::memory::Memory) -> Self {
        Self {
            mode,
            memory,
            program_counter: 0x0000_0000,
            stack_pointer: 0x4000_0000,
            instruction: 0b00000000_00000000_00000000_00000000,
            registers: [0x0; 32],
        }
    }

    /// Advances the program counter by one. Wrapping
    fn advance_pc(&mut self) {
        if self.program_counter < 0x3FFF_FFFF {
            self.program_counter += 1;
        } else {
            self.program_counter = 0;
        }
    }

    /// Advances the stack pointer by one. Wrapping
    fn advance_sp(&mut self) {
        if self.stack_pointer < 0x7FFF_FFFF {
            self.program_counter += 1;
        } else {
            self.program_counter = 0x4000_0000;
        }
    }

    /// Moves the stack pointer back by one. Wrapping
    fn decrease_sp(&mut self) {
        if self.program_counter > 0x4000_000 {
            self.program_counter -= 1;
        } else {
            self.program_counter = 0x7FFF_FFFF;
        }
    }

    fn get_instruction(&mut self, memory: &crate::memory::Memory) {
        let mut instruction: [u8; 4] = [0; 4];
        for i in 0..4 {
            instruction[i] = memory.data[self.program_counter];
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
                    println!("Ignoring error...");
                }
            }
            CpuMode::Unstable => {
                println!("{} {}", severity, error);
                println!("Ignoring error...");
            }
            CpuMode::Debug => {
                println!("{} {}", severity, error);
                println!("Program Counter: 0x{:032X}", self.program_counter);
                println!("Stack Pointer: 0x{:032X}", self.stack_pointer);
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
        self.get_instruction(self.memory);
        let opcode = TryFrom::try_from((self.instruction >> 25) & 0x7F).expect("Invalid OpCode");
        //println!("0x{:032X}: {opcode}", self.program_counter);
        match opcode {
            OpCode::LOAD => {},
            OpCode::DIV => return Err(CpuError::new(self.program_counter, CpuErrorType::DivisionByZero)),
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
#[display("CPU error occured at {program_counter:#010X}: {error_type}")]
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
}

impl Severity for CpuErrorType {
    fn is_minor(&self) -> bool {
        match self {
            CpuErrorType::StackOverflow => true,
            CpuErrorType::InvalidInstruction(_) => true,
            CpuErrorType::UnimplementedOpCode(_) => true,
            CpuErrorType::Halt => true,
            CpuErrorType::DivisionByZero => false,
        }
    }
}

pub trait Severity {
    fn is_minor(&self) -> bool;
    fn is_severe(&self) -> bool {
        !self.is_minor()
    }
}
