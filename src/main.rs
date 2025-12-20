#![allow(unused_assignments)]

#[macro_use]
extern crate derive_more;

use colored::Colorize;

use crate::opcodes::OpCode;

mod cpu;
mod memory;
mod opcodes;

fn main() {
    let mut addr: usize = 0;
    // let mut mem = std::sync::Arc::new(std::sync::Mutex::new(memory::Memory::empty()));

    let mut memory = memory::Memory::empty();

    memory.data[0x7] = (OpCode::HALT as u8) << 1;

    let mem = std::sync::Arc::new(std::sync::Mutex::new(memory));

    let mut cpu = cpu::CPU::new(cpu::CpuMode::Debug, std::sync::Arc::clone(&mem));
    println!("\nStarted VM in {} mode", format!("{}", cpu.mode).green());
    cpu.run();
}

