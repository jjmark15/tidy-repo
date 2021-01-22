use async_std::fs::File;
use async_std::path::{Path, PathBuf};

use crate::ports::persistence::filesystem::FileSystemPersistenceError;
use crate::ports::persistence::{Credentials, Persist, PersistenceError};
use crate::utils::environment::EnvironmentReader;

const HOME_ENVIRONMENT_VARIABLE: &str = "TIDY_REPO_HOME";
const CREDENTIALS_FILE_NAME: &str = "credentials.yml";

#[derive(Debug, Default)]
pub struct FileSystemCredentialsPersistence<E: EnvironmentReader> {
    environment_reader: E,
}

impl<E: EnvironmentReader> FileSystemCredentialsPersistence<E> {
    pub fn new(environment_reader: E) -> Self {
        FileSystemCredentialsPersistence { environment_reader }
    }

    async fn create_file_if_does_not_exist(
        &self,
        p: &Path,
    ) -> Result<(), FileSystemPersistenceError> {
        if !p.exists().await {
            File::create(p).await?;
        }
        Ok(())
    }

    fn credentials_file_path(&self) -> Result<PathBuf, FileSystemPersistenceError> {
        let app_home_directory =
            shellexpand::tilde(&self.environment_reader.read(HOME_ENVIRONMENT_VARIABLE)?)
                .to_string();
        Ok([app_home_directory, CREDENTIALS_FILE_NAME.to_string()]
            .iter()
            .collect())
    }

    fn serialize_data(&self, data: Credentials) -> Result<String, FileSystemPersistenceError> {
        serde_yaml::to_string(&data).map_err(FileSystemPersistenceError::from)
    }

    fn deserialize_data(&self, s: String) -> Result<Credentials, FileSystemPersistenceError> {
        serde_yaml::from_str(s.as_str()).map_err(FileSystemPersistenceError::from)
    }

    async fn write_to_file(
        &self,
        p: &Path,
        contents: String,
    ) -> Result<(), FileSystemPersistenceError> {
        async_std::fs::write(p, contents)
            .await
            .map_err(FileSystemPersistenceError::from)
    }

    async fn read_from_file(&self, p: &Path) -> Result<String, FileSystemPersistenceError> {
        async_std::fs::read_to_string(p)
            .await
            .map_err(FileSystemPersistenceError::from)
    }
}

#[async_trait::async_trait]
impl<E> Persist for FileSystemCredentialsPersistence<E>
where
    E: EnvironmentReader + Send + Sync,
{
    async fn get(&self) -> Result<Credentials, PersistenceError> {
        let path: PathBuf = self.credentials_file_path()?;
        let file_contents = self.read_from_file(path.as_path()).await?;
        self.deserialize_data(file_contents)
            .map_err(PersistenceError::from)
    }

    async fn store(&self, data: Credentials) -> Result<(), PersistenceError> {
        let path: PathBuf = self.credentials_file_path()?;
        self.create_file_if_does_not_exist(path.as_path()).await?;
        let contents = self.serialize_data(data)?;
        self.write_to_file(path.as_path(), contents).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use assert_fs::prelude::*;
    use predicates::ord::eq;
    use spectral::prelude::*;

    use crate::utils::environment::MockEnvironmentReader;

    use super::*;

    fn under_test(
        environment_reader: MockEnvironmentReader,
    ) -> FileSystemCredentialsPersistence<MockEnvironmentReader> {
        FileSystemCredentialsPersistence::new(environment_reader)
    }

    async fn read_credentials_file_contents(p: &Path) -> Result<Credentials, std::io::Error> {
        let contents = async_std::fs::read_to_string(p).await?;
        Ok(serde_yaml::from_str(contents.as_str()).unwrap())
    }

    async fn write_credentials_to_file(p: &Path, credentials: Credentials) {
        let contents = serde_yaml::to_string(&credentials).unwrap();
        async_std::fs::write(p, contents).await.unwrap()
    }

    fn mock_environment_reader(directory_path_string: String) -> MockEnvironmentReader {
        let mut mock_environment_reader = MockEnvironmentReader::default();
        mock_environment_reader
            .expect_read()
            .with(eq(HOME_ENVIRONMENT_VARIABLE))
            .returning(move |_| Ok(directory_path_string.clone()));
        mock_environment_reader
    }

    #[async_std::test]
    async fn storing_credentials_creates_file_when_not_present() {
        let temp_directory = assert_fs::TempDir::new().unwrap();
        let temp_directory_path = temp_directory.path().to_path_buf();
        let credentials_file_path = temp_directory.child(CREDENTIALS_FILE_NAME);
        let credentials = Credentials::new("token".parse().unwrap());
        let mock_environment_reader =
            mock_environment_reader(temp_directory_path.to_str().unwrap().to_string());

        under_test(mock_environment_reader)
            .store(credentials)
            .await
            .unwrap();

        credentials_file_path.assert(predicates::path::exists());
        temp_directory.close().unwrap();
    }

    #[async_std::test]
    async fn stores_credentials_as_yaml() {
        let temp_directory = assert_fs::TempDir::new().unwrap();
        let temp_directory_path = temp_directory.path().to_path_buf();
        let credentials_file_path = temp_directory.child(CREDENTIALS_FILE_NAME);
        let credentials = Credentials::new("token".parse().unwrap());
        let mock_environment_reader =
            mock_environment_reader(temp_directory_path.to_str().unwrap().to_string());

        under_test(mock_environment_reader)
            .store(credentials)
            .await
            .unwrap();

        assert_that(
            &read_credentials_file_contents(credentials_file_path.path())
                .await
                .unwrap(),
        )
        .is_equal_to(Credentials::new("token".parse().unwrap()));
        temp_directory.close().unwrap();
    }

    #[async_std::test]
    async fn loads_credentials_from_file() {
        let temp_directory = assert_fs::TempDir::new().unwrap();
        let temp_directory_path = temp_directory.path().to_path_buf();
        let credentials_file_path = temp_directory.child(CREDENTIALS_FILE_NAME);
        let credentials = Credentials::new("token".parse().unwrap());
        let mock_environment_reader =
            mock_environment_reader(temp_directory_path.to_str().unwrap().to_string());
        write_credentials_to_file(credentials_file_path.path(), credentials).await;

        assert_that(&under_test(mock_environment_reader).get().await.unwrap())
            .is_equal_to(Credentials::new("token".parse().unwrap()));
        temp_directory.close().unwrap();
    }

    #[async_std::test]
    async fn fails_to_load_credentials_from_file_when_file_does_not_exist() {
        let temp_directory = assert_fs::TempDir::new().unwrap();
        let temp_directory_path = temp_directory.path().to_path_buf();
        let mock_environment_reader =
            mock_environment_reader(temp_directory_path.to_str().unwrap().to_string());

        let result = under_test(mock_environment_reader).get().await;

        assert_that(&matches!(result.err().unwrap(), PersistenceError::FileSystemPersistence {..}))
            .is_true();
        temp_directory.close().unwrap();
    }
}
