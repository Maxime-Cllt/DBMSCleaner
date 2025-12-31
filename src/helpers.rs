use crate::colors::{BLUE, GREEN, RESET};
use crate::config::{Config, ConnectionEngine};
use crate::logger::{log_message, LogType};
use num_format::{Locale, ToFormattedString};
use std::error::Error;

/// Merge the schema into a single string
pub fn merge_schema(schema: &str) -> String {
    schema
        .split(',')
        .map(|s| format!("'{}'", s.trim()))
        .collect::<Vec<String>>()
        .join(",")
}

/// Get the url connection string based on the driver type
pub fn get_url_connection(config: &Config, schema: &str) -> Result<String, Box<dyn Error>> {
    let password = config.get_password()?;
    match config.driver {
        ConnectionEngine::Mysql | ConnectionEngine::MariaDB => Ok(format!(
            "mysql://{}:{}@{}:{}/",
            config.username, password, config.host, config.port
        )),
        ConnectionEngine::Postgres => Ok(format!(
            "postgresql://{}:{}@{}:{}/{}",
            config.username, password, config.host, config.port, schema
        )),
        ConnectionEngine::Invalid => Err("Invalid driver".into()),
    }
}

/// Print the log report when the cleaner is done
pub fn log_report(start_bytes_size: i64, end_bytes_size: i64) {
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
    log_message(&json_log, &LogType::Info);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colors::{BLUE, GREEN, RED, RESET, YELLOW};

    #[tokio::test]
    async fn test_merge_schema_single() {
        let schema = "schema1";
        let result = merge_schema(schema);
        assert_eq!(result, "'schema1'");
    }

    #[tokio::test]
    async fn test_merge_schema_multiple() {
        let schema = "schema1, schema2, schema3";
        let result = merge_schema(schema);
        assert_eq!(result, "'schema1','schema2','schema3'");
    }

    #[tokio::test]
    async fn test_merge_schema() {
        let schema: String = String::from("test");
        assert_eq!(merge_schema(&schema), "'test'");
        assert_ne!(merge_schema(&schema), " 'test' ");

        let schema: String = String::from("test,test1");
        assert_eq!(merge_schema(&schema), "'test','test1'");
        assert_ne!(merge_schema(&schema), " 'test', 'test1' ");

        let schema: String = String::from("test,test1,test2");
        assert_eq!(merge_schema(&schema), "'test','test1','test2'");
        assert_ne!(merge_schema(&schema), " 'test', 'test1', 'test2' ");

        let schema: String = String::from("test, test1, test2  ");
        assert_eq!(merge_schema(&schema), "'test','test1','test2'");
        assert_ne!(merge_schema(&schema), " 'test', 'test1', 'test2' ");

        let schema: String = String::from("    test , test1,     test2  ");
        assert_eq!(merge_schema(&schema), "'test','test1','test2'");
    }

    #[tokio::test]
    async fn test_constants() {
        assert_eq!(RED, "\x1b[31m");
        assert_eq!(GREEN, "\x1b[32m");
        assert_eq!(YELLOW, "\x1b[33m");
        assert_eq!(BLUE, "\x1b[34m");
        assert_eq!(RESET, "\x1b[0m");
    }

    #[tokio::test]
    async fn test_get_url_connection_mysql() {
        let config = Config {
            name: None,
            driver: ConnectionEngine::Mysql,
            host: "localhost".to_string(),
            port: "3306".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            password_env: None,
            schema: "test".to_string(),
        };
        let url = get_url_connection(&config, "test").unwrap();
        assert_eq!(url, "mysql://user:pass@localhost:3306/");
    }

    #[tokio::test]
    async fn test_get_url_connection_postgres() {
        let config = Config {
            name: None,
            driver: ConnectionEngine::Postgres,
            host: "localhost".to_string(),
            port: "5432".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            password_env: None,
            schema: "public".to_string(),
        };
        let url = get_url_connection(&config, "public").unwrap();
        assert_eq!(url, "postgresql://user:pass@localhost:5432/public");
    }

    #[tokio::test]
    async fn test_get_connection_url() {
        let mysql_config_test: Config = crate::config::tests::get_test_config(ConnectionEngine::Mysql, "3306");
        let postgres_config_test: Config = crate::config::tests::get_test_config(ConnectionEngine::Postgres, "5432");
        let error: Config = crate::config::tests::get_test_config(ConnectionEngine::Invalid, "5432");

        let url: String = get_url_connection(&mysql_config_test, &String::from("test")).unwrap();
        assert_eq!(url, "mysql://root:password@localhost:3306/");

        let url: String = get_url_connection(&postgres_config_test, &String::from("test")).unwrap();
        assert_eq!(url, "postgresql://root:password@localhost:5432/test");

        let url: Result<String, Box<dyn std::error::Error>> =
            get_url_connection(&error, &String::from("test"));
        assert!(url.is_err());
    }
}
