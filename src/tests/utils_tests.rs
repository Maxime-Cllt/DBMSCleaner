use crate::enums::connection_engine::ConnectionEngine;
use crate::structs::config::Config;
use crate::tests::config_test::get_test_config;
use crate::utils::constant::{BLUE, GREEN, RED, RESET, YELLOW};
use crate::utils::libcleaner::{get_url_connection, merge_schema};

#[tokio::test]
async fn test_merge_schema() {
    let schema: String = String::from("test");
    assert_eq!(merge_schema(&schema), "'test'");
    assert_ne!(merge_schema(&schema), " 'test' ");

    let schema: String = String::from("test,test1");
    assert_eq!(merge_schema(&schema), "'test','test1'");
    assert_ne!(merge_schema(&schema), " 'test', 'test1' ");

    let schema: String = String::from("test,test1,test2");
    assert_eq!(merge_schema(&schema), "'test','test1','test2'");
    assert_ne!(merge_schema(&schema), " 'test', 'test1', 'test2' ");

    let schema: String = String::from("test, test1, test2  ");
    assert_eq!(merge_schema(&schema), "'test','test1','test2'");
    assert_ne!(merge_schema(&schema), " 'test', 'test1', 'test2' ");

    let schema: String = String::from("    test , test1,     test2  ");
    assert_eq!(merge_schema(&schema), "'test','test1','test2'");
}

#[tokio::test]
async fn test_constants() {
    assert_eq!(RED, "\x1b[31m");
    assert_eq!(GREEN, "\x1b[32m");
    assert_eq!(YELLOW, "\x1b[33m");
    assert_eq!(BLUE, "\x1b[34m");
    assert_eq!(RESET, "\x1b[0m");
}

#[tokio::test]
async fn test_get_connection_url() {
    let mysql_config_test: Config = get_test_config(ConnectionEngine::Mysql, "3306");
    let postgres_config_test: Config = get_test_config(ConnectionEngine::Postgres, "5432");
    let error: Config = get_test_config(ConnectionEngine::Invalid, "5432");

    let url: String = get_url_connection(&mysql_config_test, &String::from("test")).unwrap();
    assert_eq!(url, "mysql://root:password@localhost:3306/");

    let url: String = get_url_connection(&postgres_config_test, &String::from("test")).unwrap();
    assert_eq!(url, "postgresql://root:password@localhost:5432/test");

    let url: Result<String, Box<dyn std::error::Error>> =
        get_url_connection(&error, &String::from("test"));
    assert!(url.is_err());
}
