pub struct Register<T: TryInto<T> + std::fmt::Display> {
    pub name: String,
    pub value: T,
}

impl<T> Register<T>
where
    T: std::default::Default + std::fmt::Display,
{
    pub fn create(name: &str) -> Register<T> {
        Self {
            name: name.to_string(),
            value: Default::default(),
        }
    }

    pub fn load(
        &mut self,
        val: impl TryInto<T, Error = std::num::TryFromIntError> + std::fmt::Display,
    ) -> Result<crate::logging::Message, crate::vmerror::VmError> {
        let display = val.to_string();
        self.value = val.try_into().map_err(|e: std::num::TryFromIntError| {
            crate::vmerror::VmError::External(e.to_string())
        })?;
        Ok(crate::logging::Message::Info(format!(
            "Loaded value {} into {}",
            display, self.name
        )))
    }
}
