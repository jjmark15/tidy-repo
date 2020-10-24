use crate::ports::persistence::{Credentials, PersistenceError};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait Persist {
    async fn load(&self) -> Result<Credentials, PersistenceError>;

    async fn store(&self, data: Credentials) -> Result<(), PersistenceError>;
}
