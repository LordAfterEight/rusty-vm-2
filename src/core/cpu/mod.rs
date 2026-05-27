pub mod register;
pub mod core;
pub mod opcodes;

pub struct CPU<'a> {
    pub cores : [core::Core<'a>; 4],

}

impl<'a> CPU<'a> {
    pub fn new<'b>(bus: &'a crate::core::runtime::bus::Bus) -> Self {
        let mut core = core::Core::new("Core 0", bus);
        Self {
            cores: [core; 4],
        }
    }
}