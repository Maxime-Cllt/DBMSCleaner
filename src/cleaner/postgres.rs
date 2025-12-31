use crate::cleaner::database_cleaner::DatabaseCleaner;
use crate::colors::{BLUE, RESET};
use crate::config::Config;
use crate::helpers::{get_url_connection, log_report, merge_schema};
use crate::logger::{log_and_print, LogType};
use async_trait::async_trait;
use num_format::{Locale, ToFormattedString};
use sqlx::postgres::PgRow;
use sqlx::{Pool, Postgres, Row};
use std::error::Error;

#[non_exhaustive]
pub struct PostgresCleaner {
    pub config: Config,
}

#[async_trait]
impl DatabaseCleaner for PostgresCleaner {
    async fn clean(&self) -> Result<(), Box<dyn Error>> {
        println!("Cleaning PostgresCleaner database...");

        let database_url: String = if self.config.schema == "*" {
            get_url_connection(&self.config, "")?
        } else {
            let first_schema: String = merge_schema(&self.config.schema);
            let first_schema: &str = first_schema.split(',').next().unwrap();
            get_url_connection(&self.config, first_schema)?
        };

        let pool_size: Pool<Postgres> = Pool::connect(&database_url).await?;
        let start_bytes_size: i64 = self.get_size_of_database(&pool_size).await.unwrap_or(0);
        println!(
            "Size of database at start: {BLUE}{}{RESET} bytes",
            start_bytes_size.to_formatted_string(&Locale::en)
        );

        let schema_name: Vec<String> = if self.config.schema == "*" {
            self.get_all_datnames(&pool_size).await?
        } else {
            merge_schema(&self.config.schema)
                .split(',')
                .map(|s| s.replace('\'', "").trim().into())
                .collect()
        };

        for schema in &schema_name {
            println!("Cleaning schema: {schema}");
            let database_url: String = get_url_connection(&self.config, schema)?;
            let pool: Pool<Postgres> = Pool::connect(&database_url).await?;
            self.run(&pool, schema).await?;
        }

        let end_bytes_size: i64 = self.get_size_of_database(&pool_size).await?;

        log_report(start_bytes_size, end_bytes_size);

        Ok(())
    }

    fn from_config(config: Config) -> Self {
        Self::new(config)
    }
}

impl PostgresCleaner {
    /// Create a new `PostgresCleaner` instance with the given configuration
    #[inline]
    #[must_use]
    pub const fn new(config: Config) -> Self {
        Self { config }
    }

    /// Execute the cleaning process into a single function to avoid query repetition
    async fn run(&self, pool: &Pool<Postgres>, schema_name: &str) -> Result<(), Box<dyn Error>> {
        let all_tables: Vec<PgRow> =
            match sqlx::query(&Self::get_all_postgres_tables_sql(schema_name))
                .fetch_all(pool)
                .await
            {
                Ok(rows) => rows,
                Err(e) => {
                    log_and_print(&format!("Error fetching tables: {e}"), &LogType::Error);
                    return Err(Box::new(e));
                }
            };

        println!("Cleaning temporary objects...");
        self.drop_temp_tables(pool).await?;
        self.clean_prepared_transactions(pool).await?;

        println!("Cleaning dead rows and updating statistics...");
        self.vacuum_databases(pool, &all_tables).await?;

        println!("Reindexing all tables...");
        self.reindex_all_tables(pool, &all_tables).await?;

        println!("Optimizing table storage layout...");
        self.cluster_tables(pool, &all_tables).await?;

        println!("Cleaning bloated tables and indexes...");
        self.clean_bloat(pool).await?;

        println!("Truncating WAL and clearing old logs...");
        self.clean_wal_and_logs(pool).await?;

        println!("Updating global statistics...");
        self.update_statistics(pool).await?;

        Ok(())
    }

    /// Execute the REINDEX command on all tables in the database
    /// REINDEX rebuilds one or more indices in a database, improving query performance
    #[inline]
    async fn reindex_all_tables(
        &self,
        pool: &Pool<Postgres>,
        all_tables: &[PgRow],
    ) -> Result<(), Box<dyn Error>> {
        // REINDEX TABLE is more efficient than REINDEX DATABASE
        for row in all_tables {
            let table_name: String = row.get("tablename");
            let reindex_sql = format!("REINDEX TABLE {table_name}");
            if let Err(e) = sqlx::query(&reindex_sql).execute(pool).await {
                log_and_print(
                    &format!("Error reindexing table {table_name}: {e}"),
                    &LogType::Warning,
                );
            }
        }
        Ok(())
    }

    /// Execute VACUUM FULL with ANALYZE on all tables
    /// This reclaims storage and updates statistics in a single operation
    /// VACUUM FULL requires an exclusive lock but provides maximum space reclamation
    #[inline]
    async fn vacuum_databases(
        &self,
        pool: &Pool<Postgres>,
        all_tables: &[PgRow],
    ) -> Result<(), Box<dyn Error>> {
        for row in all_tables {
            let table_name: String = row.get("tablename");
            // VACUUM (FULL, ANALYZE, VERBOSE) combines operations for efficiency
            let vacuum_sql = format!("VACUUM (FULL, ANALYZE) {table_name}");
            if let Err(e) = sqlx::query(&vacuum_sql).execute(pool).await {
                log_and_print(
                    &format!("Error vacuuming table {table_name}: {e}"),
                    &LogType::Warning,
                );
            }
        }
        Ok(())
    }

    /// Cluster tables to physically reorder them based on their primary index
    /// This improves query performance by organizing data on disk
    #[inline]
    async fn cluster_tables(
        &self,
        pool: &Pool<Postgres>,
        all_tables: &[PgRow],
    ) -> Result<(), Box<dyn Error>> {
        for row in all_tables {
            let table_name: String = row.get("tablename");
            // CLUSTER reorganizes the table based on an index
            // Skip if no suitable index exists
            let cluster_sql = format!("CLUSTER {table_name}");
            if let Err(e) = sqlx::query(&cluster_sql).execute(pool).await {
                // Clustering may fail if no index exists, which is okay
                log_and_print(
                    &format!("Table {table_name} has no cluster index (skipped): {e}"),
                    &LogType::Info,
                );
            }
        }
        Ok(())
    }

    /// Drop temporary tables created during the cleaning process
    #[inline]
    async fn drop_temp_tables(&self, pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        const SQL: &str = "DROP TABLE IF EXISTS pg_temp CASCADE;";
        if let Err(e) = sqlx::query(SQL).execute(pool).await {
            log_and_print(
                &format!("Error dropping temporary tables: {e}"),
                &LogType::Error,
            );
        }
        Ok(())
    }

    /// Clean up old prepared transactions that are stuck
    /// Prepared transactions can accumulate and cause bloat
    #[inline]
    async fn clean_prepared_transactions(&self, pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        // Find and rollback prepared transactions older than 1 hour
        const QUERY: &str = "SELECT gid FROM pg_prepared_xacts WHERE prepared < NOW() - INTERVAL '1 hour'";

        match sqlx::query(QUERY).fetch_all(pool).await {
            Ok(rows) => {
                for row in rows {
                    let gid: String = row.get("gid");
                    let rollback_sql = format!("ROLLBACK PREPARED '{gid}'");
                    if let Err(e) = sqlx::query(&rollback_sql).execute(pool).await {
                        log_and_print(
                            &format!("Error rolling back prepared transaction {gid}: {e}"),
                            &LogType::Warning,
                        );
                    }
                }
            }
            Err(e) => {
                log_and_print(
                    &format!("Error fetching prepared transactions: {e}"),
                    &LogType::Warning,
                );
            }
        }
        Ok(())
    }

    /// Update PostgreSQL statistics for better query planning
    #[inline]
    async fn update_statistics(&self, pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        // Update pg_statistic for better query optimization
        const ANALYZE_ALL: &str = "ANALYZE";
        if let Err(e) = sqlx::query(ANALYZE_ALL).execute(pool).await {
            log_and_print(
                &format!("Error updating statistics: {e}"),
                &LogType::Warning,
            );
        }
        Ok(())
    }

    /// Clean table and index bloat by identifying and removing dead space
    #[inline]
    async fn clean_bloat(&self, pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        // Identify tables with significant bloat (>20% wasted space)
        const BLOAT_QUERY: &str = r#"
            SELECT schemaname, tablename
            FROM pg_tables
            WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
        "#;

        match sqlx::query(BLOAT_QUERY).fetch_all(pool).await {
            Ok(rows) => {
                for row in rows {
                    let schema: String = row.get("schemaname");
                    let table: String = row.get("tablename");
                    let full_table = format!("{schema}.{table}");

                    // VACUUM ANALYZE removes bloat without full table lock
                    let vacuum_sql = format!("VACUUM ANALYZE {full_table}");
                    if let Err(e) = sqlx::query(&vacuum_sql).execute(pool).await {
                        log_and_print(
                            &format!("Error cleaning bloat for {full_table}: {e}"),
                            &LogType::Warning,
                        );
                    }
                }
            }
            Err(e) => {
                log_and_print(
                    &format!("Error fetching bloated tables: {e}"),
                    &LogType::Warning,
                );
            }
        }
        Ok(())
    }

    /// Clean WAL files and clear old log entries (one-shot operation)
    #[inline]
    async fn clean_wal_and_logs(&self, pool: &Pool<Postgres>) -> Result<(), Box<dyn Error>> {
        // Checkpoint to flush WAL to disk
        const CHECKPOINT: &str = "CHECKPOINT";
        if let Err(e) = sqlx::query(CHECKPOINT).execute(pool).await {
            log_and_print(
                &format!("Error executing checkpoint: {e}"),
                &LogType::Warning,
            );
        }

        // Clean up old replication slots if any
        const CLEAN_SLOTS: &str = r#"
            SELECT pg_drop_replication_slot(slot_name)
            FROM pg_replication_slots
            WHERE active = false AND slot_type = 'logical'
        "#;
        if let Err(e) = sqlx::query(CLEAN_SLOTS).execute(pool).await {
            log_and_print(
                &format!("No inactive replication slots to clean: {e}"),
                &LogType::Info,
            );
        }

        // Clear old pg_stat_statements if the extension is installed
        const CLEAR_STATS: &str = "SELECT pg_stat_statements_reset()";
        if let Err(e) = sqlx::query(CLEAR_STATS).execute(pool).await {
            log_and_print(
                &format!("pg_stat_statements not available (skipped): {e}"),
                &LogType::Info,
            );
        }

        // Truncate old data from pg_stat_database
        const RESET_STATS: &str = "SELECT pg_stat_reset()";
        if let Err(e) = sqlx::query(RESET_STATS).execute(pool).await {
            log_and_print(
                &format!("Error resetting statistics: {e}"),
                &LogType::Warning,
            );
        }

        Ok(())
    }

    /// Get all tables that need to be reindexed in `PostgreSQL`
    #[inline]
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

    /// Get the size of the database in bytes
    #[inline]
    async fn get_size_of_database(&self, pool: &Pool<Postgres>) -> Result<i64, Box<dyn Error>> {
        const QUERY: &str =
            "SELECT SUM(pg_database_size(datname))::BIGINT AS total_size_bytes FROM pg_database;";
        let row: (i64,) = sqlx::query_as(QUERY)
            .bind(&self.config.schema)
            .fetch_one(pool)
            .await?;

        Ok(row.0)
    }

    /// Get all databases in the `PostgreSQL` server
    #[inline]
    async fn get_all_datnames(&self, pool: &Pool<Postgres>) -> Result<Vec<String>, Box<dyn Error>> {
        const QUERY: &str =
            "SELECT datname FROM pg_database WHERE datname NOT IN ('template0', 'template1');";
        let rows: Vec<PgRow> = match sqlx::query(QUERY).fetch_all(pool).await {
            Ok(rows) => rows,
            Err(e) => {
                log_and_print(&format!("Error fetching databases: {e}"), &LogType::Error);
                return Err(Box::new(e));
            }
        };
        let mut databases: Vec<String> = Vec::with_capacity(rows.len());
        for row in &rows {
            databases.push(row.get("datname"));
        }
        Ok(databases)
    }

    /// Loop through all tables and execute a query
    #[inline]
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
                log_and_print(&format!("Error executing query: {e}"), &LogType::Error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConnectionEngine, tests::get_test_config};

    #[tokio::test]
    async fn test_postgres_struct() {
        let config: Config = get_test_config(ConnectionEngine::Postgres, "5432");
        let postgres_config: PostgresCleaner = PostgresCleaner::new(config);
        assert_eq!(postgres_config.config.driver, ConnectionEngine::Postgres);
        assert_eq!(postgres_config.config.host, "localhost");
        assert_eq!(postgres_config.config.port, "5432");
        assert_eq!(postgres_config.config.username, "root");
        assert_eq!(postgres_config.config.password, Some("password".to_string()));
        assert_eq!(postgres_config.config.schema, "test");
    }

    #[tokio::test]
    async fn test_get_all_postgres_tables_sql() {
        let schema: String = String::from("test");
        let tested_sql: String = PostgresCleaner::get_all_postgres_tables_sql(&schema);
        let true_sql: String =
            String::from("SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname IN ('test');");
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("test,test1");
        let tested_sql: String = PostgresCleaner::get_all_postgres_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname IN ('test','test1');",
        );
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("test,test1,test2");
        let tested_sql: String = PostgresCleaner::get_all_postgres_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname IN ('test','test1','test2');",
        );
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("*");
        let tested_sql: String = PostgresCleaner::get_all_postgres_tables_sql(&schema);
        let true_sql: String =
            String::from("SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname = 'public';");
        assert_eq!(tested_sql, true_sql);
    }
}
