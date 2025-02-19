use once_cell::sync::Lazy;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Mutex, MutexGuard};

pub struct Logger {
    log_file: File,
}

impl Logger {
    /// Create a new logger object
    pub(crate) fn new(file_path: &str) -> Self {
        let log_file: File = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();

        Self { log_file }
    }

    /// Log a message to the log file
    pub(crate) fn log(&self, message: &str) {
        let mut log_writer: BufWriter<&File> = BufWriter::new(&self.log_file);
        writeln!(log_writer, "[{}] {message}", chrono::Local::now()).unwrap_or_else(|_| {
            panic!("Failed to write to log file");
        });
    }
}

/// Static logger instance
pub static LOGGER: Lazy<Mutex<Logger>> = Lazy::new(|| Mutex::new(Logger::new("DBMSCleaner.log")));

/// Static function to log a message
pub fn log_message(message: &str) {
    let logger: MutexGuard<Logger> = LOGGER.lock().unwrap();
    logger.log(message);
}
