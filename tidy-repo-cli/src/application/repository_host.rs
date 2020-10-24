use crate::application::RepositoryUrlDto;
use crate::domain::branch::Branch;
use crate::domain::repository::RepositoryUrl;
use crate::domain::repository_host::RepositoryHost as DomainRepositoryHost;
use crate::domain::value_object::ValueObject;
use crate::ports::repository_hosting::adapters::RepositoryHost;

#[async_trait::async_trait]
impl<RepoHost> DomainRepositoryHost for RepoHost
where
    RepoHost: RepositoryHost + Send + Sync,
{
    type Err = <RepoHost as RepositoryHost>::Err;

    async fn list_branches(
        &self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<Branch>, Self::Err> {
        let repo_url_dto = RepositoryUrlDto::new(repository_url.value());
        self.list_branches(&repo_url_dto).await.map(|res| {
            res.iter()
                .map(|branch_name| Branch::new(branch_name.value().clone()))
                .collect()
        })
    }
}
