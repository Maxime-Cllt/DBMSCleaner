use crate::enums::connection_engine::ConnectionEngine;
use crate::enums::log_type::LogType;
use crate::structs::config::Config;
use crate::structs::logger::log_message;
use crate::utils::constant::{BLUE, GREEN, RESET};
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
    match config.driver {
        ConnectionEngine::Mysql | ConnectionEngine::MariaDB => Ok(format!(
            "mysql://{}:{}@{}:{}/",
            config.username, config.password, config.host, config.port
        )),
        ConnectionEngine::Postgres => Ok(format!(
            "postgresql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, schema
        )),
        _ => Err("Invalid driver".into()),
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
    log_message(&json_log, LogType::Info);
}
