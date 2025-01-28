use std::error::Error;
use async_trait::async_trait;
use crate::structs::config::Config;

#[async_trait]
pub trait DatabaseCleaner {

    /// Clean the database
    /// # Returns
    /// * A Result containing the size of the database in bytes
    /// * A Box<dyn Error> object
    async fn clean(&self) -> Result<(), Box<dyn Error>>;

    /// Load from a Config object
    /// # Arguments
    /// * `config` - A Config object
    /// # Returns
    /// * A Self object
    fn from_config(config: Config) -> Self
    where
        Self: Sized;
}
