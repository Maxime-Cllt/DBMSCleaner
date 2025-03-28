use crate::enums::log_type::LogType;
use crate::structs::logger::Logger;

#[tokio::test]
async fn test_logger() {
    const LOG_FILE: &str = "test_logger.log";
    let logger: Logger = Logger::new(LOG_FILE);
    assert!(std::path::Path::new(LOG_FILE).exists());

    logger.log("Test message", LogType::Info);

    let log_contents: String = std::fs::read_to_string(LOG_FILE).unwrap_or_default();

    assert!(log_contents.contains("INFO Test message"));

    if let Err(e) = std::fs::remove_file(LOG_FILE) {
        eprintln!("Failed to remove log file: {e}");
    }
}
