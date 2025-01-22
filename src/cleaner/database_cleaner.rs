use std::error::Error;
use async_trait::async_trait;
use crate::structs::config::Config;

#[async_trait]
pub trait DatabaseCleaner {
    async fn clean(&self) -> Result<(), Box<dyn Error>>;

    fn from_config(config: Config) -> Self
    where
        Self: Sized;
}
