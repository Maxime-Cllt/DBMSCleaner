use crate::enums::connection_engine::ConnectionEngine;
use crate::enums::log_type::LogType;

#[tokio::test]
async fn test_display() {
    assert_eq!(format!("{}", ConnectionEngine::Postgres), "Postgres");
    assert_eq!(format!("{}", ConnectionEngine::Mysql), "Mysql");
    assert_eq!(format!("{}", ConnectionEngine::MariaDB), "MariaDB");
    assert_eq!(format!("{}", ConnectionEngine::Invalid), "Invalid");
}

#[tokio::test]
async fn test_log_type_to_string() {
    assert_eq!(LogType::Info.to_string(), "INFO");
    assert_eq!(LogType::Warning.to_string(), "WARNING");
    assert_eq!(LogType::Error.to_string(), "ERROR");
    assert_eq!(LogType::Critical.to_string(), "CRITICAL");
}
