use async_trait::async_trait;

use crate::domain::branch::Branch;
use crate::domain::repository::RepositoryUrl;

#[cfg_attr(test, mockall::automock(type Err = TestRepositoryHostError;))]
#[async_trait]
pub trait RepositoryHost {
    type Err;

    async fn list_branches(&self, repository_url: &RepositoryUrl)
        -> Result<Vec<Branch>, Self::Err>;
}

#[cfg(test)]
#[derive(Debug, thiserror::Error)]
#[error("Repository client error occurred")]
pub struct TestRepositoryHostError;
