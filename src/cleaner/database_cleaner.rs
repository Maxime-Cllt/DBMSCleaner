use crate::config::Config;
use async_trait::async_trait;
use std::error::Error;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConnectionEngine;

    #[allow(dead_code)]
    struct DummyCleaner(Config);

    #[async_trait]
    impl DatabaseCleaner for DummyCleaner {
        async fn clean(&self) -> Result<(), Box<dyn Error>> {
            Ok(())
        }

        fn from_config(config: Config) -> Self {
            DummyCleaner(config)
        }
    }

    #[tokio::test]
    async fn test_dummy_cleaner() {
        let config = Config {
            driver: ConnectionEngine::Invalid,
            username: "user".to_string(),
            password: "pass".to_string(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            schema: "test".to_string(),
        };
        let cleaner = DummyCleaner::from_config(config);
        assert!(cleaner.clean().await.is_ok());
    }
}
