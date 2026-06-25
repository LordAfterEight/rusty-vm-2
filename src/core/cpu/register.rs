/// A generic register that can be used to hold any type of value that implements `Default` and `Display`
///
/// ## Example Usage
/// ```
/// let register = Register::<u8>::create("GPR");
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Register<'a, T: TryInto<T> + std::fmt::Display> {
    pub name: &'a str,
    pub value: T,
}

impl<'a, T> Register<'a, T>
where
    T: std::default::Default + std::fmt::Display + std::clone::Clone + num_traits::CheckedAdd,
{
    /// Creates a new generic Register with a name
    pub fn create(name: &'a str) -> Register<'a, T> {
        Self {
            name: name,
            value: Default::default(),
        }
    }

    /// Tries to insert an arbitrary value T into a Register. Returns an error if conversion fails
    ///
    /// ## Example Usage
    /// ```
    /// let register = Register::<u8>::create("8-bit Register");
    ///
    /// register.insert(200).unwrap(); // <-- This will succeed
    ///
    /// register.insert(256).unwrap(); // <-- This will fail and panic with a VmError::External(TryFromIntError)
    /// ```
    pub fn insert<E>(
        &mut self,
        val: impl TryInto<T, Error = E> + std::fmt::Display,
    ) -> Result<(), crate::core::runtime::error::VmError> {
        self.value = val.try_into().map_err(|e: E| {
            crate::core::runtime::error::VmError::External(std::any::type_name_of_val(&e).to_string())
        })?;
        Ok(())
    }

    /// Extract the Register's value
    ///
    /// ## Example Usage
    /// ```
    /// let register = Register::<u8>::create("8-bit Register");
    ///
    /// register.insert(200).unwrap();
    ///
    /// let val = register.extract();
    /// ```
    pub fn extract(&self) -> T {
        self.value.clone()
    }

    /// Increases the Register's internal value by 1
    pub fn checked_inc(&mut self, inc: T) -> Result<(), crate::core::runtime::error::VmError> {
        self.value = self
            .value
            .checked_add(&inc)
            .ok_or(crate::core::runtime::error::VmError::ValueTooBig)?;
        Ok(())
    }
}
