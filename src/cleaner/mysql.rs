use crate::structs::config::Config;
use sqlx::mysql::MySqlRow;
use sqlx::{MySql, Pool, Row};

pub struct MySqlCleaner {
    config: Config,
}

impl MySqlCleaner {
    pub fn from_config(config: Config) -> Self {
        MySqlCleaner { config }
    }
    pub async fn clean(&self) -> Result<(), sqlx::Error> {
        println!("Cleaning MySQL database...");
        let database_url: String = format!(
            "mysql://{}:{}@{}:{}/{}",
            self.config.username,
            self.config.password,
            self.config.host,
            self.config.port,
            self.config.schema
        );
        let pool: Pool<MySql> = Pool::connect(&database_url).await?;

        self.get_size_of_database(&pool).await?;
        // println!("Size of database: {} bytes", size);
        Ok(())
    }
    pub async fn get_size_of_database(&self, pool: &Pool<MySql>) -> Result<(), sqlx::Error> {
        const SIZE_SQL: &str = "SELECT SUM(data_length + index_length) AS 'size' \
        FROM information_schema.TABLES \
        WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');";

        let row: MySqlRow = sqlx::query(SIZE_SQL).fetch_one(pool).await?;
        println!("row: {:?}", row);

        Ok(())
    }
}
