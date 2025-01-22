use crate::utils::color::{RESET, YELLOW};
use sqlx::mysql::MySqlRow;
use sqlx::{MySql, Pool, Row};

/// Merge the schema into a single string
/// # Returns
/// * A string containing the schema
pub fn merge_schema(schema: &str) -> String {
    let vec_schema: Vec<&str> = schema.split(",").collect();
    let mut schema: String = String::new();
    let max: usize = vec_schema.len();
    for i in 0..max {
        schema.push_str(&format!("'{}'", vec_schema[i].trim()));
        if i < max - 1 {
            schema.push(',');
        }
    }
    schema
}

/// Loop through all tables and execute the specified command
/// # Arguments
/// * `pool` - A reference to a sqlx::Pool<MySql> object
/// * `all_tables` - A reference to a Vec<MySqlRow> object
/// * `command` - A reference to a String object
/// # Returns
/// * A Result containing the size of the database in bytes
pub async fn loop_and_execute_query_my_sql(
    pool: &Pool<MySql>,
    all_tables: &[MySqlRow],
    command: &str,
) {
    const QUERY_INDEX: &str = "all_tables";
    for row in all_tables {
        let table_name: String = row.get(QUERY_INDEX);
        let analyze_sql: String = format!("{command}{table_name}");
        match sqlx::query(&analyze_sql).execute(pool).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{YELLOW}Error for table {table_name}{RESET}: {e}");
            }
        }
    }
}
