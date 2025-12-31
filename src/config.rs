use crate::colors::{RED, RESET};
use serde::{Deserialize, Deserializer};
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::path::Path;

/// Represents the database connection engine type.
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// Represents the configuration for a single database connection.
#[derive(Deserialize, Debug, Clone)]
#[must_use]
pub struct DatabaseConfig {
    pub name: Option<String>,
    pub driver: ConnectionEngine,
    pub host: String,
    pub port: String,
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub password_env: Option<String>,
    pub schema: String,
}

/// Main configuration structure supporting multiple databases
#[derive(Deserialize, Debug)]
#[must_use]
pub struct CleanerConfig {
    pub databases: Vec<DatabaseConfig>,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default = "default_require_confirmation")]
    pub require_confirmation: bool,
}

fn default_require_confirmation() -> bool {
    true
}

/// Legacy Config type alias for compatibility with cleaners
pub type Config = DatabaseConfig;

impl DatabaseConfig {
    /// Get the password from config or environment variable
    pub fn get_password(&self) -> Result<String, Error> {
        if let Some(ref password) = self.password {
            return Ok(password.clone());
        }

        if let Some(ref env_var) = self.password_env {
            return env::var(env_var).map_err(|_| {
                Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "{RED}Environment variable {env_var} not found for database{RESET}"
                    ),
                )
            });
        }

        Err(Error::new(
            ErrorKind::InvalidInput,
            format!("{RED}No password or password_env specified{RESET}"),
        ))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Error> {
        let validations = [
            (self.port.parse::<i32>().is_err(), "Port must be a number"),
            (self.host.is_empty(), "Host must not be empty"),
            (self.username.is_empty(), "Username must not be empty"),
            (self.schema.is_empty(), "Schema must not be empty"),
            (self.driver == ConnectionEngine::Invalid, "Invalid driver"),
        ];

        for (condition, message) in validations {
            if condition {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("{RED}{message}{RESET}"),
                ));
            }
        }

        // Validate password is available
        self.get_password()?;

        Ok(())
    }
}

impl CleanerConfig {
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

        let config: CleanerConfig = serde_json::from_reader(reader)?;
        config.validate()?;

        Ok(config)
    }

    /// Validate all database configurations
    pub fn validate(&self) -> Result<(), Error> {
        if self.databases.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("{RED}No databases configured{RESET}"),
            ));
        }

        for (i, db_config) in self.databases.iter().enumerate() {
            let default_name = format!("Database #{}", i + 1);
            let db_name = db_config.name.as_ref().unwrap_or(&default_name);

            if let Err(e) = db_config.validate() {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("{RED}Invalid config for {db_name}: {e}{RESET}"),
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
    async fn test_database_config_validate_valid() {
        let config = DatabaseConfig {
            name: Some("Test DB".to_string()),
            driver: ConnectionEngine::Postgres,
            host: "localhost".to_string(),
            port: "5432".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            password_env: None,
            schema: "public".to_string(),
        };
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_database_config_validate_invalid_port() {
        let config = DatabaseConfig {
            name: None,
            driver: ConnectionEngine::Postgres,
            host: "localhost".to_string(),
            port: "not_a_number".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            password_env: None,
            schema: "public".to_string(),
        };
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_database_config_validate_invalid_driver() {
        let config = DatabaseConfig {
            name: None,
            driver: ConnectionEngine::Invalid,
            host: "localhost".to_string(),
            port: "5432".to_string(),
            username: "user".to_string(),
            password: Some("pass".to_string()),
            password_env: None,
            schema: "public".to_string(),
        };
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_cleaner_config_from_file() {
        const CONFIG_TEST_FILE: &str = "test_cleaner_config_from_file.json";

        generate_test_file_config(CONFIG_TEST_FILE);

        let loaded_config: CleanerConfig = CleanerConfig::from_file(CONFIG_TEST_FILE).unwrap();

        assert_eq!(loaded_config.databases.len(), 1);
        assert_eq!(loaded_config.databases[0].driver, ConnectionEngine::Mysql);
        assert_eq!(loaded_config.databases[0].host, "localhost");
        assert_eq!(loaded_config.databases[0].port, "3306");

        delete_test_file_config(CONFIG_TEST_FILE);
    }

    #[tokio::test]
    async fn test_database_config_struct() {
        let test_config: DatabaseConfig = get_test_config(ConnectionEngine::Mysql, "3306");
        assert_eq!(test_config.driver, ConnectionEngine::Mysql);
        assert_eq!(test_config.host, "localhost");
        assert_eq!(test_config.port, "3306");
        assert_eq!(test_config.username, "root");
        assert_eq!(test_config.password, Some("password".to_string()));
        assert_eq!(test_config.schema, "test");
    }

    #[tokio::test]
    async fn test_validate_multiple_drivers() {
        assert!(
            DatabaseConfig {
                name: None,
                driver: ConnectionEngine::Postgres,
                host: String::from("localhost"),
                port: String::from("3306"),
                username: String::from("root"),
                password: Some(String::from("password")),
                password_env: None,
                schema: String::from("test"),
            }
            .validate()
            .is_ok()
        );

        assert!(
            DatabaseConfig {
                name: None,
                driver: ConnectionEngine::MariaDB,
                host: String::from("localhost"),
                port: String::from("3306"),
                username: String::from("root"),
                password: Some(String::from("password")),
                password_env: None,
                schema: String::from("test"),
            }
            .validate()
            .is_ok()
        );

        assert!(
            DatabaseConfig {
                name: None,
                driver: ConnectionEngine::Invalid,
                host: String::from("localhost"),
                port: String::from("3306"),
                username: String::from("root"),
                password: Some(String::from("password")),
                password_env: None,
                schema: String::from("test"),
            }
            .validate()
            .is_err()
        );
    }

    fn generate_test_file_config(file_name: &str) {
        let mut file: File = File::create(file_name).unwrap();
        let data: &str = r#"{
        "databases": [
            {
                "driver": "mysql",
                "host": "localhost",
                "port": "3306",
                "username": "root",
                "password": "password",
                "schema": "test"
            }
        ]
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

    pub fn get_test_config(driver: ConnectionEngine, port: &str) -> DatabaseConfig {
        DatabaseConfig {
            name: None,
            driver,
            host: String::from("localhost"),
            port: String::from(port),
            username: String::from("root"),
            password: Some(String::from("password")),
            password_env: None,
            schema: String::from("test"),
        }
    }
}
