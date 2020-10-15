use async_trait::async_trait;
use surf::Client;

use crate::utils::http::{Error, Request, Response};

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait HttpClientFacade {
    async fn send(&self, req: Request) -> Result<Response, Error>;
}

#[derive(Debug)]
pub struct HttpClientFacadeImpl {
    client: Client,
}

impl HttpClientFacadeImpl {
    pub fn new(client: Client) -> Self {
        HttpClientFacadeImpl { client }
    }
}

#[async_trait]
impl HttpClientFacade for HttpClientFacadeImpl {
    async fn send(&self, req: Request) -> Result<Response, Error> {
        self.client
            .send(req)
            .await
            .map_err(Into::into)
            .map(Into::into)
    }
}
