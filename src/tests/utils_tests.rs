use crate::utils::color::{BLUE, GREEN, RED, RESET, YELLOW};
use crate::utils::libcleaner::merge_schema;

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
async fn test_color_code() {
    assert_eq!(RED, "\x1b[31m");
    assert_eq!(GREEN, "\x1b[32m");
    assert_eq!(YELLOW, "\x1b[33m");
    assert_eq!(BLUE, "\x1b[34m");
    assert_eq!(RESET, "\x1b[0m");
}
