extern crate integration_tests;

use integration_tests::*;

#[test]
fn test_concat_rs() {
    test_in_db("strings", |mut conn| {
        let result = conn
            .query("SELECT concat_rs('a','b')", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: String = row.get(0);

        assert_eq!(&col, "ab");
    });
}

#[test]
fn test_text_rs() {
    test_in_db("strings", |mut conn| {
        let result = conn
            .query("SELECT text_rs('hello world!')", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: String = row.get(0);

        assert_eq!(&col, "hello world!");
    });
}
