pub struct Core {
    pub bus: runtime::bus::Bus,
    pub registers: [crate::core::cpu::register::Register<u32>; 32],
}