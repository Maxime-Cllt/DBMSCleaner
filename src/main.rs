use libcleaner::cleaner::mysql::MySQLCleaner;
use libcleaner::cleaner::postgres::PostgresCleaner;
use libcleaner::enums::connection_engine::ConnectionEngine;
use libcleaner::enums::log_type::LogType;
use libcleaner::structs::config::Config;
use libcleaner::structs::logger::log_and_print;
use libcleaner::traits::database_cleaner::DatabaseCleaner;
use libcleaner::utils::constant::{GREEN, RESET};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let start: Instant = Instant::now();

    let config: Config = Config::from_file("cleaner.json").unwrap_or_else(|e| {
        log_and_print(&format!("{e}"), &LogType::Critical);
        panic!("Failed to read configuration file");
    });

    let cleaner: Box<dyn DatabaseCleaner> = match config.driver {
        ConnectionEngine::MariaDB | ConnectionEngine::Mysql => {
            Box::new(MySQLCleaner::from_config(config))
        }
        ConnectionEngine::Postgres => Box::new(PostgresCleaner::from_config(config)),
        ConnectionEngine::Invalid => {
            log_and_print(
                &format!("Unsupported database driver: {:?}", config.driver),
                &LogType::Critical,
            );
            panic!("Unsupported database driver");
        }
    };

    match cleaner.clean().await {
        Ok(()) => {
            println!("Cleaning completed in {GREEN}{:?}{RESET}", start.elapsed());
        }
        Err(e) => {
            log_and_print(&format!("{e}"), &LogType::Critical);
        }
    }
}
