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

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_display() {
        assert_eq!(format!("{}", ConnectionEngine::Postgres), "Postgres");
        assert_eq!(format!("{}", ConnectionEngine::Mysql), "Mysql");
        assert_eq!(format!("{}", ConnectionEngine::MariaDB), "MariaDB");
        assert_eq!(format!("{}", ConnectionEngine::Invalid), "Invalid");
    }

    #[tokio::test]
    async fn test_deserialize() {
        let postgres: ConnectionEngine = serde_json::from_str("\"postgres\"").unwrap();
        let mysql: ConnectionEngine = serde_json::from_str("\"mysql\"").unwrap();
        let mariadb: ConnectionEngine = serde_json::from_str("\"mariadb\"").unwrap();
        let invalid: ConnectionEngine = serde_json::from_str("\"invalid\"").unwrap();

        assert_eq!(postgres, ConnectionEngine::Postgres);
        assert_eq!(mysql, ConnectionEngine::Mysql);
        assert_eq!(mariadb, ConnectionEngine::MariaDB);
        assert_eq!(invalid, ConnectionEngine::Invalid);
    }
}
