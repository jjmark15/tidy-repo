use std::error::Error;

use async_trait::async_trait;

use crate::application::{BranchNameDto, RepositoryUrlDto};

#[cfg_attr(test, mockall::automock(type Err=TestRepositoryClientError;))]
#[async_trait]
pub trait RepositoryClient {
    type Err: Error;

    async fn list_branches(
        &self,
        repository_url: &RepositoryUrlDto,
    ) -> Result<Vec<BranchNameDto>, Self::Err>;
}

#[cfg(test)]
#[derive(Debug, thiserror::Error)]
#[error("Repository client error occurred")]
pub struct TestRepositoryClientError;
