use std::env::VarError;

#[cfg_attr(test, mockall::automock)]
pub trait EnvironmentReader {
    fn read(&self, key: &str) -> Result<String, EnvironmentReaderError>;
}

#[derive(Debug, Default)]
pub struct EnvironmentReaderStd;

impl EnvironmentReaderStd {
    pub fn new() -> Self {
        EnvironmentReaderStd::default()
    }
}

impl EnvironmentReader for EnvironmentReaderStd {
    fn read(&self, key: &str) -> Result<String, EnvironmentReaderError> {
        Ok(std::env::var(key)?)
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum EnvironmentReaderError {
    #[error(transparent)]
    ReadError(#[from] VarError),
}
