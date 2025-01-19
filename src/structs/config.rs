use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    driver: String,
    host: String,
    port: String,
    username: String,
    password: String,
    database: String,
}

impl Config {
    fn new(
        driver: &str,
        host: &str,
        port: &str,
        username: &str,
        password: &str,
        database: &str,
    ) -> Self {
        Config {
            driver: driver.to_string(),
            host: host.to_string(),
            port: port.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            database: database.to_string(),
        }
    }

    pub(crate) fn from_json(file_path: &str) -> Result<Self, std::io::Error> {
        let file: File = File::open(file_path)?;
        let reader: BufReader<File> = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}
