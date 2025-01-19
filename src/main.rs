use crate::cleaner::mysql::MySqlCleaner;
use crate::structs::config::Config;
use std::time::Instant;

#[cfg(test)]
mod tests;

mod cleaner;
mod structs;

#[tokio::main]
async fn main() {
    const FILE_PATH: &str = "config.json";

    let config: Config = Config::from_json(FILE_PATH).unwrap();

    println!("Starting cleaning...");

    let start: Instant = Instant::now();

    let cleaner: MySqlCleaner = MySqlCleaner::from_config(config);
    match cleaner.clean().await {
        Ok(_) => {
            let duration: std::time::Duration = start.elapsed();
            println!("Cleaning completed in {} seconds", duration.as_secs());
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
