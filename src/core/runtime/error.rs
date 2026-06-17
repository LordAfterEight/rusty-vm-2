#[derive(Debug)]
pub enum VmError {
    ValueTooBig,
    FetchError(String),
    External(String),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        _ = write!(f, "\x1b[38;2;255;50;50mError: \x1b[38;2;50;255;200m");
        match self {
            VmError::ValueTooBig => write!(f, "Value too big"),
            VmError::FetchError(s) => write!(f, "Fetch error: {}", s),
            VmError::External(s) => write!(f, "External: {}", s)
        }
    }
}

#[derive(Debug)]
pub enum VmRuntimeError {
    VmError(VmError),
}

impl From<VmError> for VmRuntimeError {
    fn from(value: VmError) -> Self {
        Self::VmError(value)
    }
}