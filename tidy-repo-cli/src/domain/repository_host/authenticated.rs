use async_trait::async_trait;
use serde::export::PhantomData;

use crate::application::RepositoryUrlDto;
use crate::domain::authentication::AuthenticationService;
use crate::domain::branch::Branch;
use crate::domain::error::RepositoryHostError;
use crate::domain::repository::RepositoryUrl;
use crate::domain::repository_host::RepositoryHostWrapper;
use crate::domain::value_object::ValueObject;
use crate::ports::repository_hosting::adapters::RepositoryHost;
use crate::ports::repository_hosting::AuthenticationCredentialValidity;

pub struct AuthenticatedRepositoryHostWrapper<RH, AS, AC>
where
    RH: RepositoryHost,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError>,
    AS: AuthenticationService<AuthenticationCredentials = AC>,
{
    repository_host: RH,
    authentication_service: AS,
    authentication_credentials_marker: PhantomData<AC>,
}

impl<RH, AS, AC> AuthenticatedRepositoryHostWrapper<RH, AS, AC>
where
    RH: RepositoryHost,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError>,
    AS: AuthenticationService<AuthenticationCredentials = AC>,
{
    pub fn new(repository_host: RH, authentication_service: AS) -> Self {
        AuthenticatedRepositoryHostWrapper {
            repository_host,
            authentication_service,
            authentication_credentials_marker: PhantomData::default(),
        }
    }
}

#[async_trait]
impl<RH, AS, AC> RepositoryHostWrapper for AuthenticatedRepositoryHostWrapper<RH, AS, AC>
where
    RH: RepositoryHost + Send + Sync,
    <RH as RepositoryHost>::AuthenticationCredentials: From<AC>,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError> + Send + Sync,
    AS: AuthenticationService<AuthenticationCredentials = AC> + Send + Sync,
    AC: Send + Sync,
{
    type AuthenticationCredentials = AC;

    async fn list_branches(
        &mut self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<Branch>, RepositoryHostError> {
        let repo_url_dto = RepositoryUrlDto::new(repository_url.value());

        if let Ok(credentials) = self
            .authentication_service
            .authentication_credentials()
            .await
        {
            self.repository_host
                .set_authentication_credentials(credentials.into())
        }

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
        _credentials: Self::AuthenticationCredentials,
    ) -> Result<AuthenticationCredentialValidity, RepositoryHostError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::application::BranchNameDto;
    use crate::domain::authentication::{AuthenticationError, MockAuthenticationService};
    use crate::ports::repository_hosting::adapters::{MockRepositoryHost, TestRepositoryHostError};

    use super::*;

    fn under_test(
        repository_host: MockRepositoryHost,
        authentication_service: MockAuthenticationService<String>,
    ) -> AuthenticatedRepositoryHostWrapper<
        MockRepositoryHost,
        MockAuthenticationService<String>,
        String,
    > {
        AuthenticatedRepositoryHostWrapper::new(repository_host, authentication_service)
    }

    fn mock_authentication_service() -> MockAuthenticationService<String> {
        MockAuthenticationService::default()
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

    fn prepare_mock_repository_host_set_authentication(
        mock_repository_host: &mut MockRepositoryHost,
        credentials: String,
    ) {
        mock_repository_host
            .expect_set_authentication_credentials()
            .with(eq(credentials))
            .return_const(());
    }

    #[async_std::test]
    async fn lists_branches_in_a_repository_when_not_authenticated_none() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host_list_branches(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![],
        );
        prepare_mock_repository_host_set_authentication(
            &mut mock_repository_host,
            "credentials".to_string(),
        );
        let mut mock_authentication_service = mock_authentication_service();
        mock_authentication_service
            .expect_authentication_credentials()
            .returning(move || Err(AuthenticationError::TestVariant));
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host, mock_authentication_service)
                .list_branches(&repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(vec![]);
    }

    #[async_std::test]
    async fn lists_branches_in_a_repository_when_not_authenticated_single() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host_list_branches(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![BranchNameDto::new("1".to_string())],
        );
        let mut mock_authentication_service = mock_authentication_service();
        mock_authentication_service
            .expect_authentication_credentials()
            .returning(move || Err(AuthenticationError::TestVariant));
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host, mock_authentication_service)
                .list_branches(&repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(vec![Branch::new("1".to_string())]);
    }

    #[async_std::test]
    async fn lists_branches_in_a_repository_when_authenticated_single() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host_list_branches(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![BranchNameDto::new("1".to_string())],
        );
        prepare_mock_repository_host_set_authentication(
            &mut mock_repository_host,
            "credentials".to_string(),
        );
        let mut mock_authentication_service = mock_authentication_service();
        mock_authentication_service
            .expect_authentication_credentials()
            .returning(move || Ok("credentials".to_string()));
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host, mock_authentication_service)
                .list_branches(&repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(vec![Branch::new("1".to_string())]);
    }

    #[async_std::test]
    async fn lists_branches_in_a_repository_when_not_authenticated_multiple() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host_list_branches(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![
                BranchNameDto::new("1".to_string()),
                BranchNameDto::new("2".to_string()),
            ],
        );
        let mut mock_authentication_service = mock_authentication_service();
        mock_authentication_service
            .expect_authentication_credentials()
            .returning(move || Err(AuthenticationError::TestVariant));
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host, mock_authentication_service)
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
    async fn errors_when_dependency_errors_when_not_authenticated() {
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_list_branches()
            .with(eq(RepositoryUrlDto::new("url".to_string())))
            .returning(move |_| Err(TestRepositoryHostError));
        let mut mock_authentication_service = mock_authentication_service();
        mock_authentication_service
            .expect_authentication_credentials()
            .returning(move || Err(AuthenticationError::NoCredentialsFound));
        let repository_url = RepositoryUrl::new("url".to_string());

        let result = under_test(mock_repository_host, mock_authentication_service)
            .list_branches(&repository_url)
            .await;

        assert_that(&matches!(result.err().unwrap(), RepositoryHostError::TestRepositoryHost {..}))
            .is_true();
    }

    #[async_std::test]
    #[should_panic]
    async fn panics_when_trying_to_validate_any_authentication_credentials() {
        let mock_repository_host = mock_repository_host();
        let mock_authentication_service = mock_authentication_service();

        under_test(mock_repository_host, mock_authentication_service)
            .validate_authentication_credentials("credentials".to_string())
            .await
            .unwrap();
    }
}
