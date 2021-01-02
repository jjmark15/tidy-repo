pub use credentials::*;
pub use error::*;

mod credentials;
mod error;
pub mod filesystem;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait Persist {
    async fn load(&self) -> Result<Credentials, PersistenceError>;

    async fn store(&self, data: Credentials) -> Result<(), PersistenceError>;
}
