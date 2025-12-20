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
    let mut mem = std::sync::Arc::new(std::sync::Mutex::new(memory::Memory::empty()));

    let mut cpu = cpu::CPU::new(cpu::CpuMode::Debug, std::sync::Arc::clone(&mem));
    println!("\nStarted VM in {} mode", format!("{}", cpu.mode).green());
    cpu.update();
}

