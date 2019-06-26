extern crate integration_tests;

use integration_tests::*;

#[test]
fn test_rust_quote_ident() {
    test_in_db("quote_ident", |conn| {
        let result = conn.query("SELECT rust_quote_ident('this is a test')", &[]).expect("query failed");
        let row = result.get(0);
        let col: String = row.get(0);
        assert_eq!(col, "\"this is a test\"");
    });
}
