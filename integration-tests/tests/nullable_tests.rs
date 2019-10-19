extern crate integration_tests;

use integration_tests::*;

#[test]
fn test_get_null() {
    test_in_db("nullable", |conn| {
        let result = conn.query("SELECT get_null()", &[]).expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: Option<i32> = row.get(0);

        assert_eq!(col, None);
    });
}

#[test]
fn test_rs_nullif() {
    test_in_db("nullable", |conn| {
        // 'a', 'b' => 'a'
        let result = conn
            .query("SELECT rs_nullif('a', 'b')", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: Option<String> = row.get(0);

        assert_eq!(col, Some("a".to_string()));

        // '-', '-' => NULL
        let result = conn
            .query("SELECT rs_nullif('-', '-')", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: Option<String> = row.get(0);

        assert_eq!(col, None);

        // 'a', NULL => 'a'
        let result = conn
            .query("SELECT rs_nullif('a', NULL)", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: Option<String> = row.get(0);

        assert_eq!(col, Some("a".to_string()));

        // NULL, '-' => NULL
        let result = conn
            .query("SELECT rs_nullif(NULL, '-')", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: Option<String> = row.get(0);

        assert_eq!(col, None);
    });
}
