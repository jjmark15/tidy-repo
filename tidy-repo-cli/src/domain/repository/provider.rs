use crate::domain::repository::{Repository, RepositoryUrl};

#[async_trait::async_trait]
pub trait RepositoryProvider {
    type Error;

    async fn get_repository(&self, url: &RepositoryUrl) -> Result<Repository, Self::Error>;
}

#[cfg(test)]
mockall::mock! {
    pub RepositoryProvider<Err: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    trait RepositoryProvider {
        type Error = Err;

        async fn get_repository(&self, url: &RepositoryUrl) -> Result<Repository, Err>;
    }
}
