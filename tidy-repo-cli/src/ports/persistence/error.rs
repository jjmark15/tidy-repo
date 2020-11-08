use crate::ports::persistence::adapters::filesystem::FileSystemPersistenceError;

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error(transparent)]
    FileSystemPersistence(#[from] FileSystemPersistenceError),
}
