use crate::enums::log_type::LogType;
use crate::utils::constant::{RED, RESET, YELLOW};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Mutex, MutexGuard};

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
