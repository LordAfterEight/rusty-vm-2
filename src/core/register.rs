pub struct Register<T: TryInto<T> + std::fmt::Display> {
    pub name: String,
    pub value: T,
}

/// A generic register that can be used to hold any type of value that implements `Default` and `Display`
/// 
/// ## Example Usage
/// ```
/// fn main() {
///     let register = Register::<u8>::create("GPR");
/// }
impl<T> Register<T>
where
    T: std::default::Default + std::fmt::Display,
{
    /// Creates a new generic Register with a name
    pub fn create(name: &str) -> Register<T> {
        Self {
            name: name.to_string(),
            value: Default::default(),
        }
    }

    /// Tries to load an arbitrary value T into a Register. Returns an error if conversion fails
    /// 
    /// ## Example Usage
    /// fn main() {
    ///     let register = Register::<u8>::create("8-bit Register");
    /// 
    ///     register.load(200);
    /// 
    ///     register.load(256); // <-- This will fail and return a VmError(TryFromIntError)
    /// }
    pub fn load(
        &mut self,
        val: impl TryInto<T, Error = std::num::TryFromIntError> + std::fmt::Display,
    ) -> Result<(), crate::core::vmerror::VmError> {
        self.value = val.try_into().map_err(|e: std::num::TryFromIntError| {
            crate::core::vmerror::VmError::External(e.to_string())
        })?;
        Ok(())
    }
}
