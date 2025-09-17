use libcleaner::cleaner::mysql::MySQLCleaner;
use libcleaner::enums::connection_engine::ConnectionEngine;
use libcleaner::structs::config::Config;
use crate::tests::config_test::get_test_config;

#[tokio::test]
async fn test_mariadb_struct() {
    let config: Config = get_test_config(ConnectionEngine::MariaDB, "3306");
    let maria_config: MySQLCleaner = MySQLCleaner::new(config);
    assert_eq!(maria_config.config.driver, ConnectionEngine::MariaDB);
    assert_eq!(maria_config.config.host, "localhost");
    assert_eq!(maria_config.config.port, "3306");
    assert_eq!(maria_config.config.username, "root");
    assert_eq!(maria_config.config.password, "password");
    assert_eq!(maria_config.config.schema, "test");
}

#[tokio::test]
async fn test_get_all_inno_db_tables_sql() {
    let schema: String = String::from("test");
    let tested_sql: String = MySQLCleaner::get_all_inno_db_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`', TABLE_SCHEMA, '`.`', TABLE_NAME, '` ENGINE=InnoDB') AS all_tables FROM information_schema.TABLES WHERE ENGINE = 'InnoDB' AND TABLE_SCHEMA IN ('test');",
    );
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("test,test1");
    let tested_sql: String = MySQLCleaner::get_all_inno_db_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`', TABLE_SCHEMA, '`.`', TABLE_NAME, '` ENGINE=InnoDB') AS all_tables FROM information_schema.TABLES WHERE ENGINE = 'InnoDB' AND TABLE_SCHEMA IN ('test','test1');",
    );
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("*");
    let tested_sql: String = MySQLCleaner::get_all_inno_db_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`', TABLE_SCHEMA, '`.`', TABLE_NAME, '` ENGINE=InnoDB') AS all_tables FROM information_schema.TABLES WHERE ENGINE = 'InnoDB' AND TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');",
    );
    assert_eq!(tested_sql, true_sql);
}

#[tokio::test]
async fn test_get_all_repair_tables_sql() {
    let schema: String = String::from("test");
    let tested_sql: String = MySQLCleaner::get_all_repair_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE ENGINE IN ('MyISAM', 'ARCHIVE', 'CSV') AND TABLE_SCHEMA IN ('test');",
    );
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("test,test1");
    let tested_sql: String = MySQLCleaner::get_all_repair_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE ENGINE IN ('MyISAM', 'ARCHIVE', 'CSV') AND TABLE_SCHEMA IN ('test','test1');",
    );
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("*");
    let tested_sql: String = MySQLCleaner::get_all_repair_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE ENGINE IN ('MyISAM', 'ARCHIVE', 'CSV') AND TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');",
    );
    assert_eq!(tested_sql, true_sql);
}

#[tokio::test]
async fn test_get_all_tables_sql() {
    let schema: String = String::from("test");
    let tested_sql: String = MySQLCleaner::get_all_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA IN ('test');",
    );
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("test,test1");
    let tested_sql: String = MySQLCleaner::get_all_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA IN ('test','test1');",
    );
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("test,test1,test2");
    let tested_sql: String = MySQLCleaner::get_all_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA IN ('test','test1','test2');",
    );
    assert_eq!(tested_sql, true_sql);

    let schema: String = String::from("*");
    let tested_sql: String = MySQLCleaner::get_all_tables_sql(&schema);
    let true_sql: String = String::from(
        "SELECT CONCAT('`',TABLE_SCHEMA,'`.`', TABLE_NAME, '`') AS all_tables FROM information_schema.TABLES WHERE TABLE_SCHEMA NOT IN ('information_schema', 'mysql', 'performance_schema', 'sys');",
    );
    assert_eq!(tested_sql, true_sql);
}
