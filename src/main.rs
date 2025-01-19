use crate::structs::config::Config;

#[cfg(test)]
mod tests;

mod structs;

fn main() {
    const FILE_PATH: &str = "config.json";

    println!("Starting cleaning...");

    let config: Config = Config::from_json(FILE_PATH).unwrap();

    println!("{:?}", config)
}
