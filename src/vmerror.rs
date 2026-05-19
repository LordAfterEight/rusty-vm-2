#[derive(Debug)]
pub enum VmError {
    ValueTooBig,
    External(String),
}
