extern crate integration_tests;

use integration_tests::*;

#[test]
fn test_concat_rs() {
    test_in_db("strings", |conn| {
        let result = conn.query("SELECT concat_rs('a','b')", &[]).expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: String = row.get(0);

        assert_eq!(&col, "ab");
    });
}
