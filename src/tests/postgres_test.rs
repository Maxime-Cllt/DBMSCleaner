use crate::cleaner::postgres::PostgresCleaner;
use crate::structs::config::Config;
use crate::tests::config_test::get_test_config;

#[tokio::test]
async fn test_postgres_struct() {
    let config: Config = get_test_config("postgres", "5432");
    let postgres_config: PostgresCleaner = PostgresCleaner::new(config);
    assert_eq!(postgres_config.config.driver, "postgres");
    assert_eq!(postgres_config.config.host, "localhost");
    assert_eq!(postgres_config.config.port, "5432");
    assert_eq!(postgres_config.config.username, "root");
    assert_eq!(postgres_config.config.password, "password");
    assert_eq!(postgres_config.config.schema, "test");
}

#[tokio::test]
async fn test_get_all_postgres_tables_sql() {
    let schema: String = String::from("test");
    let tested_sql: String = PostgresCleaner::get_all_postgres_tables_sql(&schema);
    let true_sql: String =
        String::from("SELECT datname AS all_tables FROM pg_database WHERE datname IN ('test');");
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("test,test1");
    let tested_sql: String = PostgresCleaner::get_all_postgres_tables_sql(&schema);
    let true_sql: String =
        String::from("SELECT datname AS all_tables FROM pg_database WHERE datname IN ('test','test1');");
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("test,test1,test2");
    let tested_sql: String = PostgresCleaner::get_all_postgres_tables_sql(&schema);
    let true_sql: String =
        String::from("SELECT datname AS all_tables FROM pg_database WHERE datname IN ('test','test1','test2');");
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("*");
    let tested_sql: String = PostgresCleaner::get_all_postgres_tables_sql(&schema);
    let true_sql: String =
        String::from("SELECT datname AS all_tables FROM pg_database WHERE datname NOT IN ('template0', 'template1');");
    assert_eq!(tested_sql, true_sql);
}
