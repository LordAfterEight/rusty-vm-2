#![allow(unused_assignments)]

#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate tracing;

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, EnvFilter};

use crate::opcodes::OpCode;

mod cpu;
mod core;
mod memory;
mod opcodes;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    log_file: Option<String>
}

fn main() {
    let args = Args::parse();
    let filter = EnvFilter::builder().with_default_directive(LevelFilter::TRACE.into()).from_env_lossy();
    let stdout_layer = fmt::layer().with_writer(std::io::stdout).with_filter(filter.clone());
    let log_file_path = args.log_file.unwrap_or_else(|| "log.json".to_string());
    let log_file = std::fs::File::create(log_file_path).unwrap();
    let (non_blocking,_guard) = tracing_appender::non_blocking(log_file);
    let json_layer = fmt::layer().json().with_writer(non_blocking).with_filter(filter);
    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(json_layer)
        .init();

    let mut memory = memory::Memory::empty();

    /*
    memory.data[0x0] = 0x18; // Core 0 reset addr
    memory.data[0x4] = 0x84; // Core 1 reset addr
    memory.data[0x27] = (OpCode::IRPT_SEND as u8) << 1;
    // xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx
    memory.data[0x26] = 0b00010000;
    memory.data[0x25] = 0b10000000;

    memory.data[0x87] = (OpCode::IRPT_SEND as u8) << 1;
    memory.data[0x86] = 0b00000001;
    memory.data[0x85] = 0b00000000;

    */

    memory.data[0x0]  = 0x88;

    memory.data[0x8B] = (OpCode::LDUP_IMM as u8) << 1;
    memory.data[0x8A] = 0b00011111;
    memory.data[0x89] = 0b11111111;
    memory.data[0x88] = 0b11111111;

    memory.data[0x8F] = (OpCode::ORI as u8) << 1;
    memory.data[0x8E] = 0b00011000;
    memory.data[0x8D] = 0b00001111;
    memory.data[0x8C] = 0b11111111;

    memory.data[0x93] = (OpCode::LDUP_IMM as u8) << 1;
    memory.data[0x92] = 0b00101111;
    memory.data[0x91] = 0b11111111;
    memory.data[0x90] = 0b11111111;

    memory.data[0x97] = (OpCode::ORI as u8) << 1;
    memory.data[0x96] = 0b00101000;
    memory.data[0x95] = 0b00001111;
    memory.data[0x94] = 0b11111111;

    memory.data[0x9B] = ((OpCode::BRAN_REL as u8) << 1) | 0b0;
    memory.data[0x9A] = 0b10000000;

    let mem = std::sync::Arc::new(std::sync::RwLock::new(memory));

    let mut cpu = cpu::CPU::new(cpu::CpuMode::Debug, std::sync::Arc::clone(&mem));
    info!("Started VM in {} mode", format!("{}", cpu.mode));
    cpu.run();
}
