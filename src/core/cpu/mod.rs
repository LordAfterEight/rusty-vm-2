pub mod register;
pub mod core;
pub mod opcodes;

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct CPU<'a> {
    pub cores : [core::Core<'a>; 4],

}

impl<'a> CPU<'a> {
    pub fn new() -> Self {
        let core0 = core::Core::new("Core 0");
        let core1 = core::Core::new("Core 1");
        let core2 = core::Core::new("Core 2");
        let core3 = core::Core::new("Core 3");
        Self {
            cores: [core0, core1, core2, core3],
        }
    }

    pub fn link_bus(&mut self, bus: std::sync::Arc<crate::core::runtime::bus::Bus>) {
        for core in &mut self.cores {
            core.bus = Some(bus.clone())
        }
    }
}