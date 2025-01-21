use crate::structs::config::Config;
use sqlx::mysql::MySqlRow;
use sqlx::{MySql, Pool, Row};

pub struct MariaDBCleaner {
    config: Config,
}

impl MariaDBCleaner {
    pub fn from_config(config: Config) -> Self {
        MariaDBCleaner { config }
    }
    pub async fn clean(&self) -> Result<(), sqlx::Error> {
        println!("Cleaning MariaDBCleaner database...");
        let database_url: String = format!(
            "mysql://{}:{}@{}:{}/{}",
            self.config.username,
            self.config.password,
            self.config.host,
            self.config.port,
            self.config.schema
        );
        let pool: Pool<MySql> = Pool::connect(&database_url).await?;

        let start_size: u64 = self.get_size_of_database(&pool).await?;

        self.clean_all_tables(&pool).await?;
        // println!("Size of database: {} bytes", size);
        Ok(())
    }

    pub async fn clean_all_tables(&self, pool: &Pool<MySql>) -> Result<(), sqlx::Error> {
        const SIZE_SQL: &str = "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS \
        stmt FROM information_schema.TABLES WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');";

        let rows: Vec<MySqlRow> = sqlx::query(SIZE_SQL).fetch_all(pool).await?;

        const ANALYZE_SQL: &str = "ANALYZE TABLE ";
        for row in rows {
            let table: String = row.get("stmt");
            let analyze_sql: String = format!("{ANALYZE_SQL}{table}");
            println!("Analyzing table: {}", table);
            // sqlx::query(&analyze_sql).execute(pool).await?;
        }
        Ok(())
    }

    /// Get the size of the database in bytes
    /// # Arguments
    /// * `pool` - A reference to a sqlx::Pool<MySql> object
    /// # Returns
    /// * A Result containing the size of the database in bytes
    pub async fn get_size_of_database(&self, pool: &Pool<MySql>) -> Result<u64, sqlx::Error> {
        const SIZE_SQL: &str = "SELECT CAST(SUM(data_length + index_length) AS SIGNED) AS 'size'
                                FROM information_schema.TABLES
                                WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');";

        let row: MySqlRow = sqlx::query(SIZE_SQL).fetch_one(pool).await?;
        let size: u64 = row.get("size");
        Ok(size)
    }
}
