pub mod register;
pub mod core;
pub mod opcodes;

pub struct CPU {
    pub cores : [core::Core; 4],
}