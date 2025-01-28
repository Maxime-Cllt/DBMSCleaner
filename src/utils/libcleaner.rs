
/// Merge the schema into a single string
/// # Arguments
/// * `schema` - A reference to a str object
/// # Returns
/// * A String object
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