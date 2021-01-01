use async_trait::async_trait;

#[async_trait]
pub trait PersistAuthentication {
    type AuthenticationCredentials;
    type Err;

    async fn persist_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<(), Self::Err>;

    async fn credentials(&self) -> Result<Self::AuthenticationCredentials, Self::Err>;
}

#[cfg(test)]
mockall::mock! {
    pub PersistAuthentication<AC: 'static + Send + Sync, Err: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    pub trait PersistAuthentication {
        type AuthenticationCredentials = AC;
        type Err = Err;

        async fn persist_credentials(
            &self,
            credentials: AC,
        ) -> Result<(), Err>;

        async fn credentials(
            &self,
        ) -> Result<AC, Err>;
    }
}
