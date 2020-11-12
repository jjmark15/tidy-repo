use async_trait::async_trait;
use serde::export::PhantomData;

use crate::application::RepositoryUrlDto;
use crate::domain::branch::Branch;
use crate::domain::repository::RepositoryUrl;
use crate::domain::repository_host::{RepositoryHostError, RepositoryHostWrapper};
use crate::domain::value_object::ValueObject;
use crate::ports::repository_hosting::AuthenticationCredentialValidity;
use crate::ports::repository_hosting::RepositoryHost;

pub struct UnauthenticatedRepositoryHostWrapper<RH, AC>
where
    RH: RepositoryHost,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    repository_host: RH,
    authentication_credentials_marker: PhantomData<AC>,
}

impl<RH, AC> UnauthenticatedRepositoryHostWrapper<RH, AC>
where
    RH: RepositoryHost,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    pub fn new(repository_host: RH) -> Self {
        UnauthenticatedRepositoryHostWrapper {
            repository_host,
            authentication_credentials_marker: PhantomData::default(),
        }
    }
}

#[async_trait]
impl<RH, AC> RepositoryHostWrapper for UnauthenticatedRepositoryHostWrapper<RH, AC>
where
    RH: RepositoryHost + Send + Sync,
    <RH as RepositoryHost>::AuthenticationCredentials: From<AC> + Send + Sync,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError> + Send + Sync,
    AC: Send + Sync,
{
    type AuthenticationCredentials = AC;

    async fn list_branches(
        &self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<Branch>, RepositoryHostError> {
        let repo_url_dto = RepositoryUrlDto::new(repository_url.value());

        self.repository_host
            .list_branches(&repo_url_dto)
            .await
            .map(|res| {
                res.iter()
                    .map(|branch_name| Branch::new(branch_name.value().clone()))
                    .collect()
            })
            .map_err(Into::into)
    }

    async fn validate_authentication_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<AuthenticationCredentialValidity, RepositoryHostError> {
        self.repository_host
            .validate_authentication_credentials(credentials.into())
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::application::BranchNameDto;
    use crate::ports::repository_hosting::{MockRepositoryHost, TestRepositoryHostError};

    use super::*;

    fn under_test(
        repository_host: MockRepositoryHost,
    ) -> UnauthenticatedRepositoryHostWrapper<MockRepositoryHost, String> {
        UnauthenticatedRepositoryHostWrapper::new(repository_host)
    }

    fn mock_repository_host() -> MockRepositoryHost {
        MockRepositoryHost::default()
    }

    fn prepare_mock_repository_host_list_branches(
        mock_repository_host: &mut MockRepositoryHost,
        url: RepositoryUrlDto,
        branches: Vec<BranchNameDto>,
    ) {
        mock_repository_host
            .expect_list_branches()
            .with(eq(url))
            .returning(move |_| Ok(branches.clone()));
    }

    #[async_std::test]
    async fn lists_branches_in_a_repository_none() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host_list_branches(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![],
        );
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host)
                .list_branches(&repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(vec![]);
    }

    #[async_std::test]
    async fn lists_branches_in_a_repository_single() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host_list_branches(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![BranchNameDto::new("1".to_string())],
        );
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host)
                .list_branches(&repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(vec![Branch::new("1".to_string())]);
    }

    #[async_std::test]
    async fn lists_branches_in_a_repository_multiple() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host_list_branches(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![
                BranchNameDto::new("1".to_string()),
                BranchNameDto::new("2".to_string()),
            ],
        );
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host)
                .list_branches(&repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(vec![
            Branch::new("1".to_string()),
            Branch::new("2".to_string()),
        ]);
    }

    #[async_std::test]
    async fn list_branches_fails_when_dependency_errors() {
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_list_branches()
            .with(eq(RepositoryUrlDto::new("url".to_string())))
            .returning(move |_| Err(TestRepositoryHostError));
        let repository_url = RepositoryUrl::new("url".to_string());

        let result = under_test(mock_repository_host)
            .list_branches(&repository_url)
            .await;

        assert_that(&matches!(result.err().unwrap(), RepositoryHostError::TestRepositoryHost {..}))
            .is_true();
    }

    #[async_std::test]
    async fn validates_authentication_credentials() {
        let credentials = String::from("credentials");
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_validate_authentication_credentials()
            .with(eq(credentials.clone()))
            .returning(|_| Ok(AuthenticationCredentialValidity::Valid));

        assert_that(
            &under_test(mock_repository_host)
                .validate_authentication_credentials(credentials)
                .await
                .unwrap(),
        )
        .is_equal_to(AuthenticationCredentialValidity::Valid);
    }

    #[async_std::test]
    async fn throws_error_when_validating_invalid_authentication_credentials() {
        let credentials = String::from("credentials");
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_validate_authentication_credentials()
            .with(eq(credentials.clone()))
            .returning(|_| Ok(AuthenticationCredentialValidity::Invalid));

        assert_that(
            &under_test(mock_repository_host)
                .validate_authentication_credentials(credentials)
                .await
                .unwrap(),
        )
        .is_equal_to(AuthenticationCredentialValidity::Invalid);
    }
}
