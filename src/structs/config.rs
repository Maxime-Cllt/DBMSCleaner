use crate::utils::constant::{MARIADB, MYSQL, POSTGRES, RED, RESET};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub driver: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub schema: String,
}

impl Config {
    pub fn from_json(file_path: &str) -> Result<Self, std::io::Error> {
        let path: &Path = Path::new(file_path);
        if !path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{RED}File not found: {file_path}{RESET}"),
            ));
        }
        let file: File = File::open(file_path)?;
        let reader: BufReader<File> = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;

        Config::check_config(&config).expect("Invalid configuration");

        Ok(config)
    }

    /// Check if the configuration is valid
    /// # Arguments
    /// * `config` - The configuration to check
    /// # Panics
    /// Panics if the configuration is invalid
    fn check_config(config: &Config) -> Result<(), Error> {
        let validations = [
            (config.port.parse::<i32>().is_err(), "Port must be a number"),
            (
                !(config.driver == MYSQL || config.driver == POSTGRES || config.driver == MARIADB),
                &format!("Unsupported database driver: {}", config.driver),
            ),
            (config.host.is_empty(), "Host must not be empty"),
            (config.username.is_empty(), "Username must not be empty"),
            (config.schema.is_empty(), "Schema must not be empty"),
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

    #[tokio::test]
    async fn test_check_config() {
        assert!(Config::check_config(&Config {
            driver: String::from(MYSQL),
            host: String::from("localhost"),
            port: String::from("3306"),
            username: String::from("root"),
            password: String::from("password"),
            schema: String::from("test"),
        })
        .is_ok());

        assert!(Config::check_config(&Config {
            driver: String::from("invalid"),
            host: String::from("localhost"),
            port: String::from("3306"),
            username: String::from("root"),
            password: String::from("password"),
            schema: String::from("test"),
        })
        .is_err());
    }
}
