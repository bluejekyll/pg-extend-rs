extern crate integration_tests;

use integration_tests::*;

#[test]
fn test_memory_context() {
    test_in_db("memory_context", |conn| {
        let result = conn.query("SELECT allocate()", &[]).expect("query failed");
        assert_eq!(result.len(), 1);
    });
}
