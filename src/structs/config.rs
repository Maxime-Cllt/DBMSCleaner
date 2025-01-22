use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize)]
pub struct Config {
    pub driver: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub schema: String,
}

impl Config {
    pub fn from_json(file_path: &str) -> Result<Self, std::io::Error> {
        let file: File = File::open(file_path)?;
        let reader: BufReader<File> = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}
