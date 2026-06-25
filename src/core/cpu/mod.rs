pub mod register;
pub mod core;
pub mod opcodes;

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct CPU<'a> {
    pub cores : [core::Core<'a>; 4],

}

impl<'a> CPU<'a> {
    pub fn new(bus: std::sync::Arc<crate::core::runtime::bus::Bus>) -> Self {
        let core0 = core::Core::new("Core 0", bus.clone());
        let core1 = core::Core::new("Core 1", bus.clone());
        let core2 = core::Core::new("Core 2", bus.clone());
        let core3 = core::Core::new("Core 3", bus);
        Self {
            cores: [core0, core1, core2, core3],
        }
    }
}