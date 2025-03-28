use serde::{Deserialize, Deserializer};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
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
            "postgres" => Ok(ConnectionEngine::Postgres),
            "mysql" => Ok(ConnectionEngine::Mysql),
            "mariadb" => Ok(ConnectionEngine::MariaDB),
            _ => Ok(ConnectionEngine::Invalid),
        }
    }
}

impl Display for ConnectionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionEngine::Postgres => write!(f, "Postgres"),
            ConnectionEngine::Mysql => write!(f, "Mysql"),
            ConnectionEngine::MariaDB => write!(f, "MariaDB"),
            ConnectionEngine::Invalid => write!(f, "Invalid"),
        }
    }
}
