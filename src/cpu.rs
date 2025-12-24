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
    pub cores: [Option<crate::core::Core>; 4],
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
            let mut core = crate::core::Core::new(i as u32, all_senders.clone(), own_rx, &memory);
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
            let cpu_mode = self.mode.clone();
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
                            tx.send(e).unwrap();
                            match cpu_mode {
                                CpuMode::Debug => {
                                    loop {
                                        let mut input = [0u8; 1];
                                        std::io::stdin().read_exact(&mut input).unwrap();
                                        if input[0] == b'\n' {
                                            break;
                                        }
                                    }
                                },
                                _ => {}
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

#[derive(Debug, Display, Clone)]
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
#[display("{} {} {}: {}", self.severity(), format!("CPU error occured in Core:{} at", core_index), format!("0x{:08X}", program_counter - 4), error_type)]
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
    #[display("Invalid instruction: {:#08X}", _0)]
    InvalidInstruction(u32),
    #[display("Unimplemented OpCode: {:#?}", _0)]
    UnimplementedOpCode(OpCode),
    #[display("Invalid OpCode: {}", _0)]
    InvalidOpCode(u32),
    Halt,
    DivisionByZero,
    StackOpOutOfBounds,
    AddWithOverflow,
    SubWithOverflow,
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
            CpuErrorType::InvalidOpCode(_) => CpuErrorSeverity::Severe,
            CpuErrorType::Halt => CpuErrorSeverity::Severe,
            CpuErrorType::DivisionByZero => CpuErrorSeverity::Minor,
            CpuErrorType::StackOpOutOfBounds => CpuErrorSeverity::Minor,
            CpuErrorType::AddWithOverflow => CpuErrorSeverity::Minor,
            CpuErrorType::SubWithOverflow => CpuErrorSeverity::Minor,
        }
    }
}

#[derive(Debug, Display)]
#[display("Interrupt {:?}", interrupt_type)]
pub struct Interrupt {
    pub sender_id: u32,
    pub interrupt_type: InterruptType,
}

#[derive(Debug, Display)]
pub enum InterruptType {
    Resume,
    Halt,
    SoftReset,
    HardReset,
}
