use crate::enums::log_type::LogType;
use crate::structs::config::Config;
use crate::structs::logger::log_and_print;
use crate::traits::database_cleaner::DatabaseCleaner;
use crate::utils::constant::{BLUE, RESET, YELLOW};
use crate::utils::libcleaner::{get_url_connection, log_report, merge_schema};
use async_trait::async_trait;
use num_format::{Locale, ToFormattedString};
use sqlx::mysql::MySqlRow;
use sqlx::{Executor, MySql, Pool, Row};
use std::error::Error;

#[non_exhaustive]
pub struct MySQLCleaner {
    pub config: Config,
}

#[async_trait]
impl DatabaseCleaner for MySQLCleaner {
    async fn clean(&self) -> Result<(), Box<dyn Error>> {
        let database_url: String = get_url_connection(&self.config, &self.config.schema)?;

        let pool: Pool<MySql> = Pool::connect(&database_url).await?;
        println!("Cleaning {} database...", self.config.driver);
        let start_bytes_size: i64 = Self::get_size_of_database(&pool).await.unwrap_or(0);

        println!(
            "Size of database at start: {BLUE}{}{RESET} bytes",
            start_bytes_size.to_formatted_string(&Locale::en)
        );

        println!("Reindexing all tables...");
        self.reindex_all_tables(&pool).await?;

        println!("Repairing all tables...");
        self.check_and_repair_tables(&pool).await?;

        println!("Analysing all tables...");
        self.analyse_all_tables(&pool).await?;

        println!("Clearing logs...");
        Self::clear_logs(&pool).await?;

        let end_bytes_size: i64 = Self::get_size_of_database(&pool).await.unwrap_or(0);

        log_report(start_bytes_size, end_bytes_size);

        Ok(())
    }

    fn from_config(config: Config) -> Self {
        Self::new(config)
    }
}

impl MySQLCleaner {
    /// Create a new instance of `MySQLCleaner` with the given configuration
    #[inline]
    #[must_use]
    pub const fn new(config: Config) -> Self {
        Self { config }
    }

    /// Clear the logs of the database
    #[inline]
    async fn clear_logs(pool: &Pool<MySql>) -> Result<(), Box<dyn Error>> {
        const SQL_TO_EXECUTE: [&str; 13] = [
            "FLUSH LOGS;",                                                // Flush the logs
            "PURGE BINARY LOGS BEFORE DATE_SUB(NOW(), INTERVAL 60 DAY);", // Purge old binary logs
            "FLUSH PRIVILEGES;",                                          // Reload privilege tables
            "FLUSH TABLES;",                                              // Close all tables
            "FLUSH TABLES WITH READ LOCK;", // Close all tables and lock them
            "UNLOCK TABLES;",               // Unlock tables
            "FLUSH STATUS;",                // Reset status variables
            "FLUSH QUERY CACHE;",           // Clear the query cache
            "RESET QUERY CACHE;",           // Reset query cache memory allocation
            "FLUSH HOSTS;",                 // Reset host cache (useful for connection issues)
            "FLUSH USER_RESOURCES;",        // Reset per-user resource limits
            "SET GLOBAL innodb_buffer_pool_dump_now = ON;", // Dump InnoDB buffer pool for faster reload
            "SET GLOBAL innodb_buffer_pool_load_now = ON;", // Reload InnoDB buffer pool
        ];

        for sql in &SQL_TO_EXECUTE {
            if let Err(e) = pool.execute(*sql).await {
                log_and_print(&format!("Error executing {sql}: {e}"), &LogType::Error);
            }
        }

        Ok(())
    }

    /// Execute the ALTER TABLE command on all tables with the `InnoDB` engine
    async fn reindex_all_tables(&self, pool: &Pool<MySql>) -> Result<(), Box<dyn Error>> {
        let all_tables: Vec<MySqlRow> =
            sqlx::query(&Self::get_all_inno_db_tables_sql(&self.config.schema))
                .fetch_all(pool)
                .await?;

        Self::loop_and_execute_query_my_sql(pool, &all_tables, "ALTER TABLE ").await;

        Ok(())
    }

    /// Execute the REPAIR TABLE command only if necessary
    #[inline]
    async fn check_and_repair_tables(&self, pool: &Pool<MySql>) -> Result<(), Box<dyn Error>> {
        const CHECK_TABLE_SQL: &str = "CHECK TABLE ";
        const EXTENDED_SQL: &str = " EXTENDED;";
        const REPAIR_TABLE_SQL: &str = "REPAIR TABLE ";
        const ALL_TABLES: &str = "all_tables";
        const MSG_TEXT: &str = "Msg_text";

        // Fetch the list of all tables in the schema
        let all_tables: Vec<MySqlRow> =
            sqlx::query(&Self::get_all_repair_tables_sql(&self.config.schema))
                .fetch_all(pool)
                .await?;

        for item in &all_tables {
            let table_name: String = item.get(ALL_TABLES);
            let check_sql: String = format!("{CHECK_TABLE_SQL}{table_name}{EXTENDED_SQL}");

            let result: MySqlRow = pool.fetch_one(&*check_sql).await?;
            let msg_text: String = result.get(MSG_TEXT);

            if msg_text != "OK" {
                println!("{YELLOW}Table {table_name} needs repair{RESET}");

                let repair_sql: String = format!("{REPAIR_TABLE_SQL}{table_name}{EXTENDED_SQL}");

                if let Err(e) = pool.execute(repair_sql.as_str()).await {
                    log_and_print(
                        &format!("Error repairing table {table_name}: {e}"),
                        &LogType::Warning,
                    );
                }
            }
        }

        Ok(())
    }

    /// Execute the ANALYZE TABLE command on all tables
    #[inline]
    async fn analyse_all_tables(&self, pool: &Pool<MySql>) -> Result<(), Box<dyn Error>> {
        let all_tables: Vec<MySqlRow> = self.get_tables_from_schema(pool).await?;

        Self::loop_and_execute_query_my_sql(pool, &all_tables, "ANALYZE TABLE ").await;

        Ok(())
    }

    /// Get the size of the database in bytes
    #[inline]
    async fn get_size_of_database(pool: &Pool<MySql>) -> Result<i64, Box<dyn Error>> {
        const SIZE_SQL: &str = "SELECT CAST(SUM(data_length + index_length) AS SIGNED) AS 'size'
                                FROM information_schema.TABLES
                                WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');";
        let row: MySqlRow = sqlx::query(SIZE_SQL).fetch_one(pool).await?;
        let size: Option<i64> = row.try_get("size")?;
        Ok(size.unwrap_or(0))
    }

    /// Get all tables in the specified schema
    #[inline]
    async fn get_tables_from_schema(
        &self,
        pool: &Pool<MySql>,
    ) -> Result<Vec<MySqlRow>, Box<dyn Error>> {
        let all_tables: Vec<MySqlRow> = sqlx::query(&Self::get_all_tables_sql(&self.config.schema))
            .fetch_all(pool)
            .await?;
        Ok(all_tables)
    }

    /// Get all tables in the specified schema
    #[inline]
    pub fn get_all_tables_sql(schema: &str) -> String {
        if schema == "*" {
            return String::from(
                "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');",
            );
        }
        let mut query_all_tables: String = String::from(
            "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA IN (",
        );
        query_all_tables.push_str(merge_schema(schema).as_str());
        query_all_tables.push_str(");");
        query_all_tables
    }

    /// Get all tables that need to be reindexed (`InnoDB`)
    #[inline]
    pub fn get_all_inno_db_tables_sql(schema: &str) -> String {
        if schema == "*" {
            return String::from(
                "SELECT CONCAT('`', TABLE_SCHEMA, '`.`', TABLE_NAME, '` ENGINE=InnoDB') AS all_tables FROM information_schema.TABLES WHERE ENGINE = 'InnoDB' AND TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');",
            );
        }
        let mut query_all_tables: String = String::from(
            "SELECT CONCAT('`', TABLE_SCHEMA, '`.`', TABLE_NAME, '` ENGINE=InnoDB') AS all_tables FROM information_schema.TABLES WHERE ENGINE = 'InnoDB' AND TABLE_SCHEMA IN (",
        );
        query_all_tables.push_str(merge_schema(schema).as_str());
        query_all_tables.push_str(");");
        query_all_tables
    }

    /// Get all tables that need to be repaired (`MyISAM`, `ARCHIVE`, `CSV`)
    #[inline]
    pub fn get_all_repair_tables_sql(schema: &str) -> String {
        if schema == "*" {
            return String::from(
                "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE ENGINE IN ('MyISAM', 'ARCHIVE', 'CSV') AND TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');",
            );
        }
        let mut query_all_tables: String = String::from(
            "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE ENGINE IN ('MyISAM', 'ARCHIVE', 'CSV') AND TABLE_SCHEMA IN (",
        );
        query_all_tables.push_str(merge_schema(schema).as_str());
        query_all_tables.push_str(");");
        query_all_tables
    }

    /// Loop through all tables and execute the specified command
    #[inline]
    pub async fn loop_and_execute_query_my_sql(
        pool: &Pool<MySql>,
        all_tables: &[MySqlRow],
        command: &str,
    ) {
        const QUERY_INDEX: &str = "all_tables";
        for row in all_tables {
            let table_name: String = row.get(QUERY_INDEX);
            let sql_to_execute: String = format!("{command}{table_name}");
            if let Err(e) = pool.execute(sql_to_execute.as_str()).await {
                log_and_print(
                    &format!("Error for table {table_name}: {e}"),
                    &LogType::Error,
                );
            }
        }
    }
}
