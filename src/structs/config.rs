use crate::enums::connection_engine::ConnectionEngine;
use crate::utils::constant::{RED, RESET};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub driver: ConnectionEngine,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub schema: String,
}

impl Config {
    /// Load the configuration file
    pub fn load_config(file_path: &str) -> Result<Self, Error> {
        let path: &Path = Path::new(file_path);
        if !path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("{RED}File not found: {file_path}{RESET}"),
            ));
        }
        let file: File = File::open(file_path)?;
        let reader: BufReader<File> = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;

        Config::check_config(&config).expect("Invalid configuration file");

        Ok(config)
    }

    /// Check if the configuration is valid
    pub(crate) fn check_config(config: &Config) -> Result<(), Error> {
        let validations = [
            (config.port.parse::<i32>().is_err(), "Port must be a number"),
            (config.host.is_empty(), "Host must not be empty"),
            (config.username.is_empty(), "Username must not be empty"),
            (config.schema.is_empty(), "Schema must not be empty"),
            (config.driver == ConnectionEngine::Invalid, "Invalid driver"),
        ];

        for (condition, message) in validations {
            if condition {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("{RED}{message}{RESET}"),
                ));
            }
        }
        Ok(())
    }
}
