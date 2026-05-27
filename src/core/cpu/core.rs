#[derive(Clone, Copy)]
pub struct Core<'a> {
    pub name: &'a str,
    pub bus: &'a crate::core::runtime::bus::Bus,
    pub registers: [crate::core::cpu::register::Register<'a, u32>; 32],
}

impl<'a> Core<'a> {
    pub fn new(name: &'a str, bus: &'a crate::core::runtime::bus::Bus) -> Self {
        let mut core = Self {
            name: name,
            bus,
            registers: [crate::core::cpu::register::Register::<'a, u32>::create("GPR"); 32],
        };

        core.registers[0].name = "IPR";
        core.registers[1].name = "SPR";

        core
    }
}