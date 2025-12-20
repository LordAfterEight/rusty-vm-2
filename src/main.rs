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

    let mem = std::sync::Arc::new(std::sync::Mutex::new(memory));

    let mut cpu = cpu::CPU::new(cpu::CpuMode::Debug, std::sync::Arc::clone(&mem));
    info!("Started VM in {} mode", format!("{}", cpu.mode));
    cpu.run();
}
