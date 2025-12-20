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
    pub cores: [Core; 4],
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
        let cores = std::array::from_fn(|i| Core::new(i.try_into().unwrap()));
        Self {
            mode,
            memory,
            cores,
            channel: std::sync::mpsc::channel::<CpuError>(),
        }
    }

    fn handle_errors(&self, error: CpuError) {
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
                    info!("Ignoring error...\n");
                }
            }
            CpuMode::Unstable => {
                info!("Ignoring error...\n");
            }
            CpuMode::Debug => {
                info!(
                    "Program Counter: 0x{:08X}",
                    self.cores[error.core_index as usize].program_counter
                );
                info!(
                    "Stack Pointer: 0x{:08X}",
                    self.cores[error.core_index as usize].stack_pointer
                );
                info!("Registers: {:?}", self.cores[error.core_index as usize].registers);
                loop {
                    let mut input = [0u8; 1];
                    std::io::stdin().read_exact(&mut input).unwrap();
                    if input[0] == b'\n' {
                        break;
                    }
                }
            }
        }
        if error.error_type == CpuErrorType::Halt {
            loop {}
        }
    }

    pub fn run(&mut self) {
        let mut handles = Vec::new();

        for mut core in self.cores {
            let memory = std::sync::Arc::clone(&self.memory);
            let tx = self.channel.0.clone();

            let handle = std::thread::Builder::new()
                .name(format!("RustyVM-Core-{}", core.index))
                .spawn(move || {
                    info!("Spawned thread: {}", std::thread::current().name().unwrap());
                    loop {
                        let result = {
                            let mut mem = memory.lock().unwrap();
                            core.tick(&mut mem)
                        };

                        if let Err(e) = result {
                            error!(core=core.index, severity=?e.severity(), "Core {} error: {} {}", core.index, e.severity(), e);
                            tx.send(e).unwrap();
                            break;
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

#[derive(Debug, Copy, Clone)]
pub struct Core {
    pub program_counter: u32,
    pub stack_pointer: u32,
    pub registers: [u32; 32],
    pub index: u32,
    pub busy: bool,
}

impl<'a> Core {
    pub fn new(index: u32) -> Self {
        info!("Created Core with index {index}");
        Self {
            program_counter: 0x0000_0000 + index*4,
            stack_pointer: 0x4000_0000,
            registers: [0; 32],
            index: index,
            busy: false,
        }
    }

    fn reset_soft(&mut self, memory: &mut crate::memory::Memory) {
        self.program_counter = 0x0+self.index*4;
        let new_addr = self.fetch_u32(memory);
        self.program_counter = new_addr;
        self.stack_pointer = 0x4000_0000;
    }

    fn reset_hard(&mut self, memory: &mut crate::memory::Memory) {
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

    fn write_u32_to_ram(&mut self, memory: &'a mut crate::memory::Memory, value: u32) {
        let value = value.to_le_bytes();
        for i in 0..4 {
            memory.data[self.stack_pointer as usize] = value[i];
            self.advance_sp();
        }
        info!(
            "Stored {:032b} to RAM at addresses 0x{:08X} - 0x{:08X}",
            u32::from_le_bytes(value),
            self.stack_pointer,
            self.stack_pointer + 4
        );
    }

    fn read_u32_from_ram(&mut self, memory: &'a mut crate::memory::Memory) -> u32 {
        let mut value: [u8; 4] = [0; 4];
        for i in 0..4 {
            self.decrease_sp();
            value[i] = memory.data[self.stack_pointer as usize];
        }
        info!(
            "Read u32 {:032b} from RAM at addresses 0x{:08X} - 0x{:08X}",
            u32::from_be_bytes(value),
            self.stack_pointer,
            self.stack_pointer + 4
        );
        return u32::from_be_bytes(value);
    }

    fn pop_u32_from_ram(&mut self, memory: &'a mut crate::memory::Memory) -> u32 {
        let mut value: [u8; 4] = [0; 4];
        for i in 0..4 {
            self.decrease_sp();
            value[i] = memory.data[self.stack_pointer as usize];
            memory.data[self.stack_pointer as usize] = 0;
        }
        info!(
            "Read u32 {:032b} from RAM at addresses 0x{:08X} - 0x{:08X}",
            u32::from_be_bytes(value),
            self.stack_pointer,
            self.stack_pointer + 4
        );
        return u32::from_be_bytes(value);
    }

    fn fetch_u32(&mut self, memory: &'a mut crate::memory::Memory) -> u32{
        let mut instruction: [u8; 4] = [0; 4];
        for i in 0..4 {
            instruction[i] = memory.data[self.program_counter as usize];
            self.advance_pc();
        }
        u32::from_le_bytes(instruction)
    }

    fn tick(&mut self, memory: &mut crate::memory::Memory) -> Result<(), CpuError> {
        self.busy = true;
        let instruction = self.fetch_u32(memory);
        let opcode_val = (instruction >> 25) & 0x7F;
        let opcode = match TryFrom::try_from(opcode_val) {
            Ok(opcode) => opcode,
            Err(_) => {
                error!(
                    core=self.index,
                    "Failed to decode OpCode at 0x{:08X}",
                    self.program_counter - 4
                );
                OpCode::NOOP
            }
        };
        info!(
            core=self.index,
            "0x{:08X}: 0x{:02X} - {}",
            self.program_counter - 4,
            opcode_val,
            opcode
        );
        match opcode {
            OpCode::LOAD_IMM => {
                let register = (instruction >> 20) & 0x1F;
                let value = instruction & 0xFFFFF;
                self.registers[register as usize] = value;
                info!("Loaded value {} into register {}", value, register);
            }
            OpCode::JUMP_IMM => {
                let addr = instruction & 0x1FFFFFF;
                self.program_counter = addr;
            }
            OpCode::JUMP_REG => {
                let register = instruction & 0x1F;
                self.program_counter = self.registers[register as usize];
            }
            OpCode::BRAN_IMM => {
                self.write_u32_to_ram(memory, self.program_counter as u32);
                let addr = instruction & 0x1FFFFFF;
                self.program_counter = addr;
            }
            OpCode::BRAN_REG => {
                self.write_u32_to_ram(memory, self.program_counter as u32);
                let register = instruction & 0x1F;
                self.program_counter = self.registers[register as usize];
            }
            OpCode::RTRN => {
                let addr = self.read_u32_from_ram(memory);
                self.program_counter = addr;
            }
            OpCode::RTRN_POP => {
                let addr = self.pop_u32_from_ram(memory);
                self.program_counter = addr;
            }
            OpCode::NOOP => {
                self.busy = false;
            }
            OpCode::RSET_SOFT => self.reset_soft(memory),
            OpCode::RSET_HARD => self.reset_hard(memory),
            OpCode::HALT => {
                return Err(CpuError::new(
                    self.program_counter,
                    CpuErrorType::Halt,
                    self.index,
                ));
            }
            _ => {
                return Err(CpuError::new(
                    self.program_counter,
                    CpuErrorType::UnimplementedOpCode(opcode),
                    self.index,
                ));
            }
        }
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
    pub core_index: u32,
}

impl CpuError {
    pub fn new(program_counter: u32, error_type: CpuErrorType, core_index: u32) -> Self {
        Self {
            error_type,
            program_counter,
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
        }
    }
}

pub trait Severity {
    fn severity(&self) -> CpuErrorSeverity;
}
