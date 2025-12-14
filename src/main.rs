use crate::opcodes::OpCode;

#[macro_use]
extern crate derive_more;

mod cpu;
mod memory;
mod opcodes;

macro_rules! size_check {
    ($op:expr, $rde:expr, $rs1:expr, $rs2:expr, $imm:expr) => {
        println!("op={} rde={} rs1={} rs2={} imm={}", $op, $rde, $rs1, $rs2, $imm);
        if ($op as u8) > 127 {panic!("Invalid OpCode: {}", $op as u8)}
        if $rde > 31 {panic!("Destination register index too high: rde = {}", $rde)}
        if $rs1 > 31 {panic!("Source register 1 index too high: rs1 = {}", $rs1)}
        if $rs2 > 31 {panic!("Source register 2 index too high: rs2 = {}", $rs2)}
        if $imm > 1023 {panic!("Immediate value exceeded limit for this instruction structure: imm = {}", $imm)}
    };
    ($op:expr, $rde:expr, $rs1:expr, $rs2:expr) => {
        println!("op={} rde={} rs1={} rs2={}", $op, $rde, $rs1, $rs2);
        if ($op as u32) > 127 {panic!("Invalid OpCode: {}", $op as u8)}
        if $rde > 31 {panic!("Destination register index too high: rde = {}", $rde)}
        if $rs1 > 31 {panic!("Source register 1 index too high: rs1 = {}", $rs1)}
        if $rs2 > 31 {panic!("Source register 2 index too high: rs2 = {}", $rs2)}
    };
    ($op:expr, $rde:expr, $imm:expr) => {
        println!("op={} rde={} imm={}", $op, $rde, $imm);
        if ($op as u32) > 127 {panic!("Invalid OpCode: {}", $op as u8)}
        if $rde > 31 {panic!("Destination register index too high: rde = {}", $rde)}
        if $imm > 1048575 {panic!("Immediate value exceeded limit for this instruction structure: imm = {}", $imm)}
    };
    ($op:expr, $imm:expr) => {
        println!("op={} imm={}", $op, $imm);
        if ($op as u32) > 127 {panic!("Invalid OpCode: {}", $op as u8)}
        if $imm > 33554431 {panic!("Immediate value exceeded limit for this instruction structure: imm = {}", $imm)}
    };
}


fn main() {
    let mut addr: usize = 0;
    let mut mem = memory::Memory::empty();

    macro_rules! insert {
        ($instr:expr) => {
            mem.data[0+addr..4+addr].copy_from_slice(&$instr.to_le_bytes());
            println!("{:032b}", $instr);
            addr += 4;
        };
    }
    macro_rules! rvmasm {
        ($op:expr, $rde:expr, $rs1:expr, $rs2:expr, $imm:expr) => {
            size_check!($op, $rde, $rs1, $rs2, $imm);
            insert!((($op as u32) << 32-7) | ($rde << 32-12) | ($rs1 << 32-17) | ($rs2 << 32-22) | ($imm << 32-27));
        };
        ($op:expr, $rde:expr, $rs1:expr, $rs2:expr) => {
            size_check!($op, $rde, $rs1, $rs2);
            insert!((($op as u32) << 32-7) | ($rde << 32-12) | ($rs1 << 32-17) | ($rs2 << 32-22));
        };
        ($op:expr, $rde:expr, $imm:expr) => {
            size_check!($op, $rde, $imm);
            insert!((($op as u32) << 32-7) | ($rde << 32-12) | ($imm));
        };
        ($op:expr, $imm:expr) => {
            size_check!($op, $imm);
            insert!((($op as u32) << 32-7) | ($imm));
        };
    }

    rvmasm!(OpCode::LOAD, 1, 2);
    rvmasm!(OpCode::JUMP, 0x1FF_FFFF);
    rvmasm!(OpCode::DIV, 1, 2, 3);

    let mut cpu = cpu::CPU::new(cpu::CpuMode::Debug, &mem);
    println!("\nStarted VM in {} mode", cpu.mode);
    loop {
        cpu.update();
    }
}

