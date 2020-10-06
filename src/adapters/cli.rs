pub trait ClientOptions {
    fn repository_urls(&self) -> &Vec<String>;
}
