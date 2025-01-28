use crate::cleaner::database_cleaner::DatabaseCleaner;
use crate::cleaner::mariadb::MariaDBCleaner;
use crate::cleaner::postgres::PostgresCleaner;
use crate::structs::config::Config;
use crate::utils::color::{GREEN, RED, RESET};
use std::time::Instant;

mod cleaner;
mod structs;
#[cfg(test)]
mod tests;

mod utils;

#[tokio::main]
async fn main() {
    const FILE_PATH: &str = "config.json";

    let config: Config = Config::from_json(FILE_PATH).unwrap();

    let start: Instant = Instant::now();

    let cleaner: Box<dyn DatabaseCleaner> = match config.driver.as_str() {
        "mariadb" | "mysql" => Box::new(MariaDBCleaner::from_config(config)),
        "postgres" => Box::new(PostgresCleaner::from_config(config)),
        _ => {
            eprintln!("{RED}Unsupported database driver: {}{RESET}", config.driver);
            std::process::exit(1);
        }
    };

    match cleaner.clean().await {
        Ok(_) => {
            println!("Cleaning completed in {GREEN}{:?}{RESET}", start.elapsed());
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
