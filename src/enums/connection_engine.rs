use serde::{Deserialize, Deserializer};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
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
        let invalid_empty: ConnectionEngine = serde_json::from_str("\"\"").unwrap();

        assert_eq!(postgres, ConnectionEngine::Postgres);
        assert_eq!(mysql, ConnectionEngine::Mysql);
        assert_eq!(mariadb, ConnectionEngine::MariaDB);
        assert_eq!(invalid, ConnectionEngine::Invalid);
        assert_eq!(invalid_empty, ConnectionEngine::Invalid);
    }
}
