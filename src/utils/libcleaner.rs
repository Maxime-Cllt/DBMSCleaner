use crate::structs::config::Config;
use crate::utils::constant::{MARIADB, MYSQL, POSTGRES};

/// Merge the schema into a single string
pub fn merge_schema(schema: &str) -> String {
    schema
        .split(',')
        .map(|s| format!("'{}'", s.trim()))
        .collect::<Vec<String>>()
        .join(",")
}

/// Get the url connection string based on the driver type
pub fn get_url_connection(
    config: &Config,
    schema: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match config.driver.as_str() {
        MYSQL | MARIADB => Ok(format!(
            "mysql://{}:{}@{}:{}/",
            config.username, config.password, config.host, config.port
        )),
        POSTGRES => Ok(format!(
            "postgresql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, schema
        )),
        _ => Err("Invalid driver".into()),
    }
}
