use crate::ports::persistence::credentials::Credentials;
use crate::ports::persistence::error::PersistenceError;

mod credentials;
mod error;
pub mod filesystem;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait Persist {
    async fn get(&self) -> Result<Credentials, PersistenceError>;

    async fn store(&self, data: Credentials) -> Result<(), PersistenceError>;
}
