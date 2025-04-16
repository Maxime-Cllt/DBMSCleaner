use crate::cleaner::database_cleaner::DatabaseCleaner;
use crate::cleaner::mysql::MySQLCleaner;
use crate::cleaner::postgres::PostgresCleaner;
use crate::enums::connection_engine::ConnectionEngine;
use crate::enums::log_type::LogType;
use crate::structs::config::Config;
use crate::structs::logger::log_and_print;
use crate::utils::constant::{GREEN, RESET};
use std::time::Instant;

mod cleaner;
mod enums;
mod structs;
#[cfg(test)]
mod tests;
mod utils;

#[tokio::main]
async fn main() {
    let start: Instant = Instant::now();

    let config: Config = Config::load_config("cleaner.json").unwrap_or_else(|e| {
        log_and_print(&format!("{e}"), LogType::Critical);
        std::process::exit(1);
    });

    let cleaner: Box<dyn DatabaseCleaner> = match config.driver {
        ConnectionEngine::MariaDB | ConnectionEngine::Mysql => {
            Box::new(MySQLCleaner::from_config(config))
        }
        ConnectionEngine::Postgres => Box::new(PostgresCleaner::from_config(config)),
        ConnectionEngine::Invalid => {
            log_and_print(
                &format!("Unsupported database driver: {:?}", config.driver),
                LogType::Critical,
            );
            std::process::exit(1);
        }
    };

    match cleaner.clean().await {
        Ok(_) => {
            println!("Cleaning completed in {GREEN}{:?}{RESET}", start.elapsed());
        }
        Err(e) => {
            log_and_print(&format!("{e}"), LogType::Critical);
        }
    }
}
