use std::marker::PhantomData;

use async_std::fs::File;

#[cfg(test)]
use crate::ports::persistence::credentials::Credentials;
use crate::ports::persistence::filesystem::FileSystemPersistenceError;
use async_std::path::Path;
use std::path::PathBuf;

#[cfg_attr(test, mockall::automock(type Content = Credentials;))]
#[async_trait::async_trait]
pub trait ContentStore {
    type Content;

    async fn get(&self) -> Result<Self::Content, FileSystemPersistenceError>;

    async fn store(&self, content: Self::Content) -> Result<(), FileSystemPersistenceError>;
}

#[derive(Debug, Default)]
pub struct SerializableContentFilesystemStore<C>
where
    C: serde::Serialize + serde::de::DeserializeOwned,
{
    content_type_marker: PhantomData<C>,
    filepath: PathBuf,
}

impl<C> SerializableContentFilesystemStore<C>
where
    C: serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(filepath: PathBuf) -> Self {
        SerializableContentFilesystemStore {
            filepath,
            content_type_marker: Default::default(),
        }
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

    fn serialize_content(&self, data: C) -> Result<String, FileSystemPersistenceError> {
        serde_yaml::to_string(&data).map_err(FileSystemPersistenceError::from)
    }

    fn deserialize_content(&self, s: String) -> Result<C, FileSystemPersistenceError> {
        serde_yaml::from_str(s.as_str()).map_err(FileSystemPersistenceError::from)
    }

    async fn write(&self, p: &Path, contents: String) -> Result<(), FileSystemPersistenceError> {
        async_std::fs::write(p, contents)
            .await
            .map_err(FileSystemPersistenceError::from)
    }

    async fn read(&self, p: &Path) -> Result<String, FileSystemPersistenceError> {
        async_std::fs::read_to_string(p)
            .await
            .map_err(FileSystemPersistenceError::from)
    }
}

#[async_trait::async_trait]
impl<C> ContentStore for SerializableContentFilesystemStore<C>
where
    C: serde::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    type Content = C;

    async fn get(&self) -> Result<Self::Content, FileSystemPersistenceError> {
        let file_contents = self.read(self.filepath.as_path().as_ref()).await?;
        self.deserialize_content(file_contents)
    }

    async fn store(&self, data: Self::Content) -> Result<(), FileSystemPersistenceError> {
        self.create_file_if_does_not_exist(self.filepath.as_path().as_ref())
            .await?;
        let contents = self.serialize_content(data)?;
        self.write(self.filepath.as_path().as_ref(), contents)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use assert_fs::prelude::*;
    use spectral::prelude::*;

    use crate::ports::persistence::credentials::Credentials;

    use super::*;

    const TEST_STORE_FILE_NAME: &str = "filename.yml";

    fn under_test(filepath: PathBuf) -> SerializableContentFilesystemStore<Credentials> {
        SerializableContentFilesystemStore::new(filepath)
    }

    async fn read_credentials_file_contents(p: &Path) -> Result<Credentials, std::io::Error> {
        let contents = async_std::fs::read_to_string(p).await?;
        Ok(serde_yaml::from_str(contents.as_str()).unwrap())
    }

    async fn write_credentials_to_file(p: &Path, credentials: Credentials) {
        let contents = serde_yaml::to_string(&credentials).unwrap();
        async_std::fs::write(p, contents).await.unwrap()
    }

    #[async_std::test]
    async fn storing_credentials_creates_file_when_not_present() {
        let temp_directory = assert_fs::TempDir::new().unwrap();
        let credentials_file_path = temp_directory.child(TEST_STORE_FILE_NAME);
        let credentials = Credentials::new("token".parse().unwrap());

        under_test(credentials_file_path.path().to_path_buf())
            .store(credentials)
            .await
            .unwrap();

        credentials_file_path.assert(predicates::path::exists());
        temp_directory.close().unwrap();
    }

    #[async_std::test]
    async fn stores_credentials_as_yaml() {
        let temp_directory = assert_fs::TempDir::new().unwrap();
        let credentials_file_path = temp_directory.child(TEST_STORE_FILE_NAME);
        let credentials = Credentials::new("token".parse().unwrap());

        under_test(credentials_file_path.path().to_path_buf())
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
        let credentials_file_path = temp_directory.child(TEST_STORE_FILE_NAME);
        let credentials = Credentials::new("token".parse().unwrap());
        write_credentials_to_file(credentials_file_path.path(), credentials).await;

        assert_that(
            &under_test(credentials_file_path.path().to_path_buf())
                .get()
                .await
                .unwrap(),
        )
        .is_equal_to(Credentials::new("token".parse().unwrap()));
        temp_directory.close().unwrap();
    }

    #[async_std::test]
    async fn fails_to_load_credentials_from_file_when_file_does_not_exist() {
        let temp_directory = assert_fs::TempDir::new().unwrap();
        let credentials_file_path = temp_directory.child(TEST_STORE_FILE_NAME);

        let result = under_test(credentials_file_path.path().to_path_buf())
            .get()
            .await;

        assert_that(&matches!(result.err().unwrap(), FileSystemPersistenceError::IO {..}))
            .is_true();
        temp_directory.close().unwrap();
    }
}
