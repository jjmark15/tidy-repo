pub mod application;
pub mod domain;
pub mod ports;
pub mod utils;

#[async_trait::async_trait]
pub trait TidyRepoApp {
    async fn run(&mut self);
}
