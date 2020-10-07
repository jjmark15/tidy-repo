use crate::utils::http::{Error, Request, Response};
use async_trait::async_trait;
use surf::Client;

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
        match self.client.send(req).await {
            Ok(surf_response) => Ok(surf_response.into()),
            Err(err) => Err(err.into()),
        }
    }
}
