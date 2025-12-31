use crate::colors::{RED, RESET};
use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::path::Path;

/// Represents the database connection engine type.
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ConnectionEngine {
    Postgres,
    Mysql,
    MariaDB,
    Invalid,
}

impl<'de> Deserialize<'de> for ConnectionEngine {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "postgres" => Ok(Self::Postgres),
            "mysql" => Ok(Self::Mysql),
            "mariadb" => Ok(Self::MariaDB),
            _ => Ok(Self::Invalid),
        }
    }
}

impl Display for ConnectionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Postgres => write!(f, "Postgres"),
            Self::Mysql => write!(f, "Mysql"),
            Self::MariaDB => write!(f, "MariaDB"),
            Self::Invalid => write!(f, "Invalid"),
        }
    }
}

/// Represents the configuration for the database connection.
#[derive(Deserialize, Debug)]
#[must_use]
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
    pub fn from_file(file_path: &str) -> Result<Self, Error> {
        let path: &Path = Path::new(file_path);
        if !path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("{RED}File not found: {file_path}{RESET}"),
            ));
        }
        let file: File = File::open(file_path)?;
        let reader: BufReader<File> = BufReader::new(file);
        let config: Self = serde_json::from_reader(reader)?;

        Self::check_config(&config).expect("Invalid configuration file");

        Ok(config)
    }

    /// Check if the configuration is valid
    pub fn check_config(config: &Self) -> Result<(), Error> {
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_connection_engine_display() {
        assert_eq!(format!("{}", ConnectionEngine::Postgres), "Postgres");
        assert_eq!(format!("{}", ConnectionEngine::Mysql), "Mysql");
        assert_eq!(format!("{}", ConnectionEngine::MariaDB), "MariaDB");
        assert_eq!(format!("{}", ConnectionEngine::Invalid), "Invalid");
    }

    #[tokio::test]
    async fn test_connection_engine_deserialize() {
        let postgres: ConnectionEngine = serde_json::from_str("\"postgres\"").unwrap();
        let mysql: ConnectionEngine = serde_json::from_str("\"mysql\"").unwrap();
        let mariadb: ConnectionEngine = serde_json::from_str("\"mariadb\"").unwrap();
        let invalid: ConnectionEngine = serde_json::from_str("\"invalid\"").unwrap();
        let invalid_empty: ConnectionEngine = serde_json::from_str("\"\"").unwrap();

        assert_eq!(postgres, ConnectionEngine::Postgres);
        assert_eq!(mysql, ConnectionEngine::Mysql);
        assert_eq!(mariadb, ConnectionEngine::MariaDB);
        assert_eq!(invalid, ConnectionEngine::Invalid);
        assert_eq!(invalid_empty, ConnectionEngine::Invalid);
    }

    #[tokio::test]
    async fn test_config_check_valid() {
        let config = Config {
            driver: ConnectionEngine::Postgres,
            host: "localhost".to_string(),
            port: "5432".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            schema: "public".to_string(),
        };
        assert!(Config::check_config(&config).is_ok());
    }

    #[tokio::test]
    async fn test_config_check_invalid_port() {
        let config = Config {
            driver: ConnectionEngine::Postgres,
            host: "localhost".to_string(),
            port: "not_a_number".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            schema: "public".to_string(),
        };
        assert!(Config::check_config(&config).is_err());
    }

    #[tokio::test]
    async fn test_config_check_invalid_driver() {
        let config = Config {
            driver: ConnectionEngine::Invalid,
            host: "localhost".to_string(),
            port: "5432".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            schema: "public".to_string(),
        };
        assert!(Config::check_config(&config).is_err());
    }

    #[tokio::test]
    async fn test_config_from_file() {
        const CONFIG_TEST_FILE: &str = "test_config_from_file.json";

        generate_test_file_config(CONFIG_TEST_FILE);

        let loaded_config: Config = Config::from_file(CONFIG_TEST_FILE).unwrap();

        assert_eq!(loaded_config.driver, ConnectionEngine::Mysql);
        assert_eq!(loaded_config.host, "localhost");
        assert_eq!(loaded_config.port, "3306");
        assert_eq!(loaded_config.username, "root");
        assert_eq!(loaded_config.password, "password");
        assert_eq!(loaded_config.schema, "test");

        delete_test_file_config(CONFIG_TEST_FILE);
    }

    #[tokio::test]
    async fn test_config_struct() {
        let test_config: Config = get_test_config(ConnectionEngine::Mysql, "3306");
        assert_eq!(test_config.driver, ConnectionEngine::Mysql);
        assert_eq!(test_config.host, "localhost");
        assert_eq!(test_config.port, "3306");
        assert_eq!(test_config.username, "root");
        assert_eq!(test_config.password, "password");
        assert_eq!(test_config.schema, "test");
    }

    #[tokio::test]
    async fn test_check_config_multiple_drivers() {
        assert!(
            Config::check_config(&Config {
                driver: ConnectionEngine::Postgres,
                host: String::from("localhost"),
                port: String::from("3306"),
                username: String::from("root"),
                password: String::from("password"),
                schema: String::from("test"),
            })
            .is_ok()
        );

        assert!(
            Config::check_config(&Config {
                driver: ConnectionEngine::MariaDB,
                host: String::from("localhost"),
                port: String::from("3306"),
                username: String::from("root"),
                password: String::from("password"),
                schema: String::from("test"),
            })
            .is_ok()
        );

        assert!(
            Config::check_config(&Config {
                driver: ConnectionEngine::Invalid,
                host: String::from("localhost"),
                port: String::from("3306"),
                username: String::from("root"),
                password: String::from("password"),
                schema: String::from("test"),
            })
            .is_err()
        );
    }

    fn generate_test_file_config(file_name: &str) {
        let mut file: File = File::create(file_name).unwrap();
        let data: &str = r#"{
        "driver": "mysql",
        "host": "localhost",
        "port": "3306",
        "username": "root",
        "password": "password",
        "schema": "test"
    }"#;

        file.write_all(data.as_bytes()).unwrap();
    }

    fn delete_test_file_config(file_name: &str) {
        match std::fs::remove_file(file_name) {
            Ok(()) => {}
            Err(_) => {
                eprintln!("Error deleting file {file_name}");
            }
        }
    }

    pub fn get_test_config(driver: ConnectionEngine, port: &str) -> Config {
        Config {
            driver,
            host: String::from("localhost"),
            port: String::from(port),
            username: String::from("root"),
            password: String::from("password"),
            schema: String::from("test"),
        }
    }
}
