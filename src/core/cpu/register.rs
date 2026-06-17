#[derive(Clone, Copy, Debug)]
pub struct Register<'a, T: TryInto<T> + std::fmt::Display> {
    pub name: &'a str,
    pub value: T,
}

/// A generic register that can be used to hold any type of value that implements `Default` and `Display`
/// 
/// ## Example Usage
/// ```
/// let register = Register::<u8>::create("GPR");
/// ```
impl<'a, T> Register<'a, T>
where
    T: std::default::Default + std::fmt::Display,
{
    /// Creates a new generic Register with a name
    pub fn create(name: &'a str) -> Register<'a, T> {
        Self {
            name: name,
            value: Default::default(),
        }
    }

    /// Tries to load an arbitrary value T into a Register. Returns an error if conversion fails
    /// 
    /// ## Example Usage
    /// ```
    /// let register = Register::<u8>::create("8-bit Register");
    /// register.load(255); // <-- This will succeed
    /// register.load(256); // <-- This will fail and return a VmError::External(TryFromIntError)
    /// ```
    pub fn load<E: std::fmt::Display>(
        &mut self,
        val: impl TryInto<T, Error = E> + std::fmt::Display,
    ) -> Result<(), crate::core::runtime::error::VmError> {
        self.value = val.try_into().map_err(|e: E| {
            crate::core::runtime::error::VmError::External(e.to_string())
        })?;
        Ok(())
    }
}
