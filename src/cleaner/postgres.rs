use crate::cleaner::database_cleaner::DatabaseCleaner;
use crate::structs::config::Config;
use crate::structs::logger::log_message;
use crate::utils::constant::{BLUE, GREEN, RED, RESET};
use crate::utils::libcleaner::{get_url_connection, merge_schema};
use async_trait::async_trait;
use num_format::{Locale, ToFormattedString};
use sqlx::postgres::PgRow;
use sqlx::{Pool, Postgres, Row};
use std::error::Error;

pub struct PostgresCleaner {
    pub config: Config,
}

#[async_trait]
impl DatabaseCleaner for PostgresCleaner {
    async fn clean(&self) -> Result<(), Box<dyn Error>> {
        println!("Cleaning PostgresCleaner database...");

        let schema_name: Vec<String> = merge_schema(&self.config.schema)
            .split(',')
            .map(|s| s.replace("'", "").trim().to_string())
            .collect();

        let database_url: String = get_url_connection(&self.config, &schema_name[0])?;
        let pool_size: Pool<Postgres> = Pool::connect(&database_url).await?;
        let start_bytes_size: i64 = self.get_size_of_database(&pool_size).await?;
        println!(
            "Size of database at start: {BLUE}{}{RESET} bytes",
            start_bytes_size.to_formatted_string(&Locale::en)
        );

        for schema in &schema_name {
            println!("Cleaning schema: {schema}");
            let database_url: String = get_url_connection(&self.config, schema)?;
            let pool: Pool<Postgres> = Pool::connect(&database_url).await?;
            self.run(&pool, schema).await?;
        }

        self.print_report(start_bytes_size, &pool_size).await?;
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

    /// Execute the cleaning process into a single function to avoid query repetition
    async fn run(&self, pool: &Pool<Postgres>, schema_name: &str) -> Result<(), Box<dyn Error>> {
        let all_tables: Vec<PgRow> = sqlx::query(&Self::get_all_postgres_tables_sql(schema_name))
            .fetch_all(pool)
            .await?;

        println!("Drop temporary tables...");
        self.drop_temp_tables(pool).await?;
        println!("Reindexing all tables...");
        self.reindex_all_database(pool, &all_tables).await?;
        println!("Vacuuming all tables...");
        self.vacuum_databases(pool, &all_tables).await?;
        println!("Analyzing all tables...");
        self.analyze_tables(pool, &all_tables).await?;

        Ok(())
    }

    /// Execute the REINDEX command on all tables in the database
    /// REINDEX rebuilds one or more indices in a database
    async fn reindex_all_database(
        &self,
        pool: &Pool<Postgres>,
        all_tables: &[PgRow],
    ) -> Result<(), Box<dyn Error>> {
        Self::loop_and_execute_query_postgres(pool, all_tables, "REINDEX DATABASE ").await;
        Ok(())
    }

    /// Execute the VACUUM command on all tables in the database
    /// VACUUM reclaims storage occupied by dead tuples. In normal PostgreSQL operation, tuples that are deleted or obsoleted by an update are not physically removed from their table; they remain present until a VACUUM is done.
    async fn vacuum_databases(
        &self,
        pool: &Pool<Postgres>,
        all_database: &[PgRow],
    ) -> Result<(), Box<dyn Error>> {
        Self::loop_and_execute_query_postgres(pool, all_database, "VACUUM FULL ").await;
        Ok(())
    }

    /// Execute the ANALYZE command on all tables in the database
    /// ANALYZE is used to update statistics used by the PostgreSQL query planner to determine the most efficient way to execute a query
    async fn analyze_tables(
        &self,
        pool: &Pool<Postgres>,
        all_tables: &[PgRow],
    ) -> Result<(), Box<dyn Error>> {
        Self::loop_and_execute_query_postgres(pool, all_tables, "ANALYZE ").await;
        Ok(())
    }

    /// Drop temporary tables created during the cleaning process
    async fn drop_temp_tables(&self, pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        const SQL: &str = "DROP TABLE IF EXISTS pg_temp CASCADE;";
        if let Err(e) = sqlx::query(SQL).execute(pool).await {
            eprintln!("{RED}Error dropping temporary tables: {e}{RESET}");
            log_message(&format!("Error dropping temporary tables: {e}"));
        }
        Ok(())
    }

    /// Get all tables that need to be reindexed in PostgreSQL
    /// # Arguments
    /// * `schema` - A reference to a String object
    /// # Returns
    /// * A String object containing the SQL query
    pub fn get_all_postgres_tables_sql(schema: &str) -> String {
        if schema == "*" {
            return String::from(
                "SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname = 'public';",
            );
        }
        let mut query_all_tables: String =
            String::from("SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname IN (");
        query_all_tables.push_str(merge_schema(schema).as_str());
        query_all_tables.push_str(");");
        query_all_tables
    }

    async fn get_size_of_database(&self, pool: &Pool<Postgres>) -> Result<i64, Box<dyn Error>> {
        const QUERY: &str =
            "SELECT SUM(pg_database_size(datname))::BIGINT AS total_size_bytes FROM pg_database;";
        let row: (i64,) = sqlx::query_as(QUERY)
            .bind(&self.config.schema)
            .fetch_one(pool)
            .await?;

        Ok(row.0)
    }

    /// Print the report of the cleaning process
    async fn print_report(
        &self,
        start_bytes_size: i64,
        pool: &Pool<Postgres>,
    ) -> Result<(), Box<dyn Error>> {
        let end_bytes_size: i64 = self.get_size_of_database(pool).await.unwrap_or(0);
        let diff: i64 = if start_bytes_size > end_bytes_size {
            start_bytes_size - end_bytes_size
        } else {
            0
        };
        println!(
            "Size of database at end: {BLUE}{}{RESET} bytes",
            end_bytes_size.to_formatted_string(&Locale::en)
        );
        println!(
            "Size of database reduced by: {GREEN}{}{RESET} bytes",
            diff.to_formatted_string(&Locale::en)
        );

        let json_log: String = format!(
            r#"{{"from_bytes": {start_bytes_size},"to_bytes": {end_bytes_size},"diff": {diff}}}"#,
        );
        log_message(&json_log);
        Ok(())
    }

    pub async fn loop_and_execute_query_postgres(
        pool: &Pool<Postgres>,
        all_tables: &[PgRow],
        command: &str,
    ) {
        const QUERY_INDEX: &str = "all_tables";
        for row in all_tables {
            let table_name: String = row.get(QUERY_INDEX);
            let analyze_sql: String = format!("{command}{table_name}");
            if let Err(e) = sqlx::query(&analyze_sql).execute(pool).await {
                eprintln!("{RED}Error executing query: {e}{RESET}");
                log_message(&format!("Error executing query: {e}"));
            }
        }
    }
}
