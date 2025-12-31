use crate::colors::{RED, RESET, YELLOW};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Mutex, MutexGuard};

/// Enum representing different types of logs
#[repr(u8)]
pub enum LogType {
    Info,
    Warning,
    Error,
    Critical,
}

impl LogType {
    /// Returns the string representation of the log type
    #[inline]
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
        }
    }
}

impl std::fmt::Display for LogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Logger struct to handle logging functionality
#[must_use]
pub struct Logger {
    log_file: File,
}

impl Logger {
    /// Create a new logger object
    pub fn new(file_path: &str) -> Self {
        let log_file: File = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();

        Self { log_file }
    }

    /// Log a message to the log file
    pub fn log(&self, message: &str, log_type: &LogType) {
        let mut log_writer: BufWriter<&File> = BufWriter::new(&self.log_file);
        writeln!(
            log_writer,
            "[{}] {} {message}",
            chrono::Local::now(),
            log_type.as_str()
        )
        .unwrap_or_else(|_| {
            panic!("Failed to write to log file");
        });
    }
}

/// Static logger instance
pub static LOGGER: std::sync::LazyLock<Mutex<Logger>> =
    std::sync::LazyLock::new(|| Mutex::new(Logger::new("DBMSCleaner.log")));

/// Static function to log a message
pub fn log_message(message: &str, log_type: &LogType) {
    let logger: MutexGuard<Logger> = LOGGER.lock().unwrap();
    logger.log(message, log_type);
}

/// Static function to log a message and print it to the console
pub fn log_and_print(message: &str, log_type: &LogType) {
    match log_type {
        LogType::Critical | LogType::Error => eprintln!("{RED}{message}{RESET}"),
        LogType::Warning => println!("{YELLOW}{message}{RESET}"),
        LogType::Info => println!("{message}"),
    }
    log_message(message, log_type);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_type_to_string() {
        assert_eq!(LogType::Info.as_str(), "INFO");
        assert_eq!(LogType::Warning.as_str(), "WARNING");
        assert_eq!(LogType::Error.as_str(), "ERROR");
        assert_eq!(LogType::Critical.as_str(), "CRITICAL");
    }

    #[tokio::test]
    async fn test_log_type_display() {
        assert_eq!(format!("{}", LogType::Info), "INFO");
        assert_eq!(format!("{}", LogType::Warning), "WARNING");
        assert_eq!(format!("{}", LogType::Error), "ERROR");
        assert_eq!(format!("{}", LogType::Critical), "CRITICAL");
    }

    #[tokio::test]
    async fn test_logger_creation() {
        let _logger = Logger::new("test.log");
        assert!(std::path::Path::new("test.log").exists());
        std::fs::remove_file("test.log").ok();
    }

    #[tokio::test]
    async fn test_logger() {
        const LOG_FILE: &str = "test_logger.log";
        let logger: Logger = Logger::new(LOG_FILE);
        assert!(std::path::Path::new(LOG_FILE).exists());

        logger.log("Test message", &LogType::Info);

        let log_contents: String = std::fs::read_to_string(LOG_FILE).unwrap_or_default();

        assert!(log_contents.contains("INFO Test message"));

        if let Err(e) = std::fs::remove_file(LOG_FILE) {
            eprintln!("Failed to remove log file: {e}");
        }
    }
}
