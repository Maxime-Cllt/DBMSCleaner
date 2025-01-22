use crate::cleaner::database_cleaner::DatabaseCleaner;
use crate::structs::config::Config;
use crate::utils::color::{BLUE, RESET};
use async_trait::async_trait;
use num_format::{Locale, ToFormattedString};
use sqlx::{Pool, Postgres};
use std::error::Error;

pub struct PostgresCleaner {
    pub config: Config,
}

#[async_trait]
impl DatabaseCleaner for PostgresCleaner {
    async fn clean(&self) -> Result<(), Box<dyn Error>> {
        println!("Cleaning PostgresCleaner database...");
        let database_url: String = format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.config.username,
            self.config.password,
            self.config.host,
            self.config.port,
            self.config.schema
        );

        let pool: Pool<Postgres> = Pool::connect(&database_url).await?;

        let start_size: i64 = self.get_size_of_database(&pool).await?;
        println!(
            "Size of database at start: {BLUE}{}{RESET} bytes",
            start_size.to_formatted_string(&Locale::en)
        );

        Ok(())
    }

    fn from_config(config: Config) -> Self {
        Self::new(config)
    }
}

impl PostgresCleaner {
    pub fn new(config: Config) -> Self {
        PostgresCleaner { config }
    }

    async fn get_size_of_database(&self, pool: &Pool<Postgres>) -> Result<i64, Box<dyn Error>> {
        const QUERY: &str =
            "SELECT SUM(pg_database_size(datname)) AS total_size_bytes FROM pg_database;";
        let row: (i64,) = sqlx::query_as(QUERY)
            .bind(&self.config.schema)
            .fetch_one(pool)
            .await?;

        Ok(row.0)
    }
}
