use crate::ports::persistence::filesystem::FileSystemPersistenceError;

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error(transparent)]
    FileSystemPersistence(#[from] FileSystemPersistenceError),
}
