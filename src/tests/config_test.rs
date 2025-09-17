use crate::utils::constant::RESET;
use crate::utils::constant::RED;
use crate::enums::connection_engine::ConnectionEngine;
use crate::structs::config::Config;
use std::fs::File;
use std::io::Write;

#[tokio::test]
async fn test_struct_from_file() {
    const CONFIG_TEST_FILE: &str = "test_struct_from_file.json";

    generate_test_file_config(CONFIG_TEST_FILE);

    let loaded_config: Config = Config::from_file(CONFIG_TEST_FILE).unwrap();

    println!("{loaded_config:?}");

    assert_eq!(loaded_config.driver, ConnectionEngine::Mysql);
    assert_eq!(loaded_config.host, "localhost");
    assert_eq!(loaded_config.port, "3306");
    assert_eq!(loaded_config.username, "root");
    assert_eq!(loaded_config.password, "password");
    assert_eq!(loaded_config.schema, "test");

    delete_test_file_config(CONFIG_TEST_FILE);
}

#[tokio::test]
async fn test_struct() {
    let test_config: Config = get_test_config(ConnectionEngine::Mysql, "3306");
    assert_eq!(test_config.driver, ConnectionEngine::Mysql);
    assert_eq!(test_config.host, "localhost");
    assert_eq!(test_config.port, "3306");
    assert_eq!(test_config.username, "root");
    assert_eq!(test_config.password, "password");
    assert_eq!(test_config.schema, "test");
}

#[tokio::test]
async fn test_check_config() {
    assert!(
        Config::check_config(&Config {
            driver: ConnectionEngine::Postgres,
            host: String::from("localhost"),
            port: String::from("3306"),
            username: String::from("root"),
            password: String::from("password"),
            schema: String::from("test"),
        })
        .is_ok()
    );

    assert!(
        Config::check_config(&Config {
            driver: ConnectionEngine::MariaDB,
            host: String::from("localhost"),
            port: String::from("3306"),
            username: String::from("root"),
            password: String::from("password"),
            schema: String::from("test"),
        })
        .is_ok()
    );

    assert!(
        Config::check_config(&Config {
            driver: ConnectionEngine::Postgres,
            host: String::from("localhost"),
            port: String::from("3306"),
            username: String::from("root"),
            password: String::from("password"),
            schema: String::from("test"),
        })
        .is_ok()
    );

    assert!(
        Config::check_config(&Config {
            driver: ConnectionEngine::Invalid,
            host: String::from("localhost"),
            port: String::from("3306"),
            username: String::from("root"),
            password: String::from("password"),
            schema: String::from("test"),
        })
        .is_err()
    );
}

fn generate_test_file_config(file_name: &str) {
    let mut file: File = File::create(file_name).unwrap();
    let data: &str = r#"{
        "driver": "mysql",
        "host": "localhost",
        "port": "3306",
        "username": "root",
        "password": "password",
        "schema": "test"
    }"#;

    file.write_all(data.as_bytes()).unwrap();
}

fn delete_test_file_config(file_name: &str) {
    match std::fs::remove_file(file_name) {
        Ok(()) => {}
        Err(_) => {
            println!("{RED}Error deleting file{file_name}{RESET}",);
        }
    }
}

pub fn get_test_config(driver: ConnectionEngine, port: &str) -> Config {
    Config {
        driver,
        host: String::from("localhost"),
        port: String::from(port),
        username: String::from("root"),
        password: String::from("password"),
        schema: String::from("test"),
    }
}
