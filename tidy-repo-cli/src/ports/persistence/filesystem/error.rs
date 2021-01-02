use crate::utils::environment::EnvironmentReaderError;

#[derive(Debug, thiserror::Error)]
pub enum FileSystemPersistenceError {
    #[error("could not serialize content")]
    Serialization(#[from] serde_yaml::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Environment(#[from] EnvironmentReaderError),
}
