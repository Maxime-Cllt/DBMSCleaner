use crate::cleaner::database_cleaner::DatabaseCleaner;
use crate::colors::{BLUE, RESET, YELLOW};
use crate::config::Config;
use crate::helpers::{get_url_connection, log_report, merge_schema};
use crate::logger::{log_and_print, LogType};
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

        println!("Cleaning temporary tables and connections...");
        self.clean_temporary_objects(&pool).await?;

        println!("Optimizing all tables (defrag + analyze + repair)...");
        self.optimize_all_tables(&pool).await?;

        println!("Checking and repairing tables if needed...");
        self.check_and_repair_tables(&pool).await?;

        println!("Rebuilding indexes for InnoDB tables...");
        self.reindex_all_tables(&pool).await?;

        println!("Updating table statistics...");
        self.analyse_all_tables(&pool).await?;

        println!("Flushing caches and logs...");
        Self::flush_caches(&pool).await?;

        println!("Purging old binary and slow query logs...");
        Self::purge_logs(&pool).await?;

        println!("Resetting statistics...");
        Self::reset_statistics(&pool).await?;

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

    /// Clean temporary tables and kill sleeping connections
    #[inline]
    async fn clean_temporary_objects(&self, pool: &Pool<MySql>) -> Result<(), Box<dyn Error>> {
        // Drop temporary tables
        const DROP_TEMP: &str = "DROP TEMPORARY TABLE IF EXISTS temp_tables";
        if let Err(e) = pool.execute(DROP_TEMP).await {
            log_and_print(
                &format!("No temporary tables to drop: {e}"),
                &LogType::Info,
            );
        }

        // Kill sleeping connections older than 1 hour
        const KILL_QUERY: &str = r#"
            SELECT CONCAT('KILL ', id, ';') AS kill_cmd
            FROM information_schema.processlist
            WHERE command = 'Sleep' AND time > 3600
        "#;

        match sqlx::query(KILL_QUERY).fetch_all(pool).await {
            Ok(rows) => {
                for row in rows {
                    let kill_cmd: String = row.get("kill_cmd");
                    if let Err(e) = pool.execute(kill_cmd.as_str()).await {
                        log_and_print(
                            &format!("Error killing connection: {e}"),
                            &LogType::Warning,
                        );
                    }
                }
            }
            Err(e) => {
                log_and_print(
                    &format!("Error fetching sleeping connections: {e}"),
                    &LogType::Info,
                );
            }
        }

        Ok(())
    }

    /// OPTIMIZE TABLE - combines defragmentation, analyze, and repair
    #[inline]
    async fn optimize_all_tables(&self, pool: &Pool<MySql>) -> Result<(), Box<dyn Error>> {
        let all_tables: Vec<MySqlRow> = self.get_tables_from_schema(pool).await?;

        for row in &all_tables {
            let table_name: String = row.get("all_tables");
            let optimize_sql = format!("OPTIMIZE TABLE {table_name}");

            if let Err(e) = pool.execute(optimize_sql.as_str()).await {
                log_and_print(
                    &format!("Error optimizing table {table_name}: {e}"),
                    &LogType::Warning,
                );
            }
        }

        Ok(())
    }

    /// Flush caches and buffers (one-shot operation)
    #[inline]
    async fn flush_caches(pool: &Pool<MySql>) -> Result<(), Box<dyn Error>> {
        const FLUSH_COMMANDS: [&str; 8] = [
            "FLUSH TABLES;",          // Close all tables
            "FLUSH HOSTS;",           // Reset host cache
            "FLUSH STATUS;",          // Reset status variables
            "FLUSH USER_RESOURCES;",  // Reset per-user resource limits
            "FLUSH QUERY CACHE;",     // Clear the query cache (if enabled)
            "RESET QUERY CACHE;",     // Reset query cache memory
            "FLUSH PRIVILEGES;",      // Reload privilege tables
            "FLUSH LOGS;",            // Flush all logs
        ];

        for cmd in &FLUSH_COMMANDS {
            if let Err(e) = pool.execute(*cmd).await {
                log_and_print(
                    &format!("Error executing {cmd}: {e}"),
                    &LogType::Warning,
                );
            }
        }

        Ok(())
    }

    /// Purge old logs (binary logs and slow query logs)
    #[inline]
    async fn purge_logs(pool: &Pool<MySql>) -> Result<(), Box<dyn Error>> {
        // Purge binary logs older than 7 days
        const PURGE_BINARY: &str = "PURGE BINARY LOGS BEFORE DATE_SUB(NOW(), INTERVAL 7 DAY)";
        if let Err(e) = pool.execute(PURGE_BINARY).await {
            log_and_print(
                &format!("Error purging binary logs: {e}"),
                &LogType::Info,
            );
        }

        // Truncate slow query log table if it exists
        const TRUNCATE_SLOW_LOG: &str = "TRUNCATE TABLE mysql.slow_log";
        if let Err(e) = pool.execute(TRUNCATE_SLOW_LOG).await {
            log_and_print(
                &format!("Slow query log table not available: {e}"),
                &LogType::Info,
            );
        }

        // Truncate general log table if it exists
        const TRUNCATE_GENERAL_LOG: &str = "TRUNCATE TABLE mysql.general_log";
        if let Err(e) = pool.execute(TRUNCATE_GENERAL_LOG).await {
            log_and_print(
                &format!("General log table not available: {e}"),
                &LogType::Info,
            );
        }

        Ok(())
    }

    /// Reset performance schema and statistics
    #[inline]
    async fn reset_statistics(pool: &Pool<MySql>) -> Result<(), Box<dyn Error>> {
        // Reset performance schema statistics
        const RESET_COMMANDS: [&str; 3] = [
            "TRUNCATE TABLE performance_schema.events_statements_summary_by_digest",
            "TRUNCATE TABLE performance_schema.events_waits_summary_global_by_event_name",
            "TRUNCATE TABLE performance_schema.file_summary_by_instance",
        ];

        for cmd in &RESET_COMMANDS {
            if let Err(e) = pool.execute(*cmd).await {
                log_and_print(
                    &format!("Performance schema table not available: {e}"),
                    &LogType::Info,
                );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConnectionEngine, tests::get_test_config};

    #[tokio::test]
    async fn test_mariadb_struct() {
        let config: Config = get_test_config(ConnectionEngine::MariaDB, "3306");
        let maria_config: MySQLCleaner = MySQLCleaner::new(config);
        assert_eq!(maria_config.config.driver, ConnectionEngine::MariaDB);
        assert_eq!(maria_config.config.host, "localhost");
        assert_eq!(maria_config.config.port, "3306");
        assert_eq!(maria_config.config.username, "root");
        assert_eq!(maria_config.config.password, Some("password".to_string()));
        assert_eq!(maria_config.config.schema, "test");
    }

    #[tokio::test]
    async fn test_get_all_inno_db_tables_sql() {
        let schema: String = String::from("test");
        let tested_sql: String = MySQLCleaner::get_all_inno_db_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`', TABLE_SCHEMA, '`.`', TABLE_NAME, '` ENGINE=InnoDB') AS all_tables FROM information_schema.TABLES WHERE ENGINE = 'InnoDB' AND TABLE_SCHEMA IN ('test');",
        );
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("test,test1");
        let tested_sql: String = MySQLCleaner::get_all_inno_db_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`', TABLE_SCHEMA, '`.`', TABLE_NAME, '` ENGINE=InnoDB') AS all_tables FROM information_schema.TABLES WHERE ENGINE = 'InnoDB' AND TABLE_SCHEMA IN ('test','test1');",
        );
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("*");
        let tested_sql: String = MySQLCleaner::get_all_inno_db_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`', TABLE_SCHEMA, '`.`', TABLE_NAME, '` ENGINE=InnoDB') AS all_tables FROM information_schema.TABLES WHERE ENGINE = 'InnoDB' AND TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');",
        );
        assert_eq!(tested_sql, true_sql);
    }

    #[tokio::test]
    async fn test_get_all_repair_tables_sql() {
        let schema: String = String::from("test");
        let tested_sql: String = MySQLCleaner::get_all_repair_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE ENGINE IN ('MyISAM', 'ARCHIVE', 'CSV') AND TABLE_SCHEMA IN ('test');",
        );
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("test,test1");
        let tested_sql: String = MySQLCleaner::get_all_repair_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE ENGINE IN ('MyISAM', 'ARCHIVE', 'CSV') AND TABLE_SCHEMA IN ('test','test1');",
        );
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("*");
        let tested_sql: String = MySQLCleaner::get_all_repair_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE ENGINE IN ('MyISAM', 'ARCHIVE', 'CSV') AND TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');",
        );
        assert_eq!(tested_sql, true_sql);
    }

    #[tokio::test]
    async fn test_get_all_tables_sql() {
        let schema: String = String::from("test");
        let tested_sql: String = MySQLCleaner::get_all_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA IN ('test');",
        );
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("test,test1");
        let tested_sql: String = MySQLCleaner::get_all_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA IN ('test','test1');",
        );
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("test,test1,test2");
        let tested_sql: String = MySQLCleaner::get_all_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA IN ('test','test1','test2');",
        );
        assert_eq!(tested_sql, true_sql);

        let schema: String = String::from("*");
        let tested_sql: String = MySQLCleaner::get_all_tables_sql(&schema);
        let true_sql: String = String::from(
            "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');",
        );
        assert_eq!(tested_sql, true_sql);
    }
}
