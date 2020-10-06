use crate::adapters::cli::ClientOptions;

mod adapters;
pub mod application;
mod domain;
pub mod ports;

pub struct TidyRepoClient<CO: ClientOptions> {
    _client_options: CO,
}

impl<CO: ClientOptions> TidyRepoClient<CO> {
    pub fn new(client_options: CO) -> Self {
        TidyRepoClient {
            _client_options: client_options,
        }
    }
}
