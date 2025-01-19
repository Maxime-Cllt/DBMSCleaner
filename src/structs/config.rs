use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub driver: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub schema: String,
}

impl Config {
    fn new(
        driver: &str,
        host: &str,
        port: &str,
        username: &str,
        password: &str,
        schema: &str,
    ) -> Self {
        Config {
            driver: driver.to_string(),
            host: host.to_string(),
            port: port.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            schema: schema.to_string(),
        }
    }

    pub fn from_json(file_path: &str) -> Result<Self, std::io::Error> {
        let file: File = File::open(file_path)?;
        let reader: BufReader<File> = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}
