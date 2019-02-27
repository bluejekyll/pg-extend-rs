extern crate integration_tests;

use integration_tests::*;

#[test]
fn test_add_one() {
    test_in_db("adding", |conn| {
        let result = conn.query("SELECT add_one(1)", &[]).expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: i32 = row.get(0);

        assert_eq!(col, 2);
    });
}

#[test]
fn test_add_small_one() {
    test_in_db("adding", |conn| {
        let result = conn
            .query("SELECT add_small_one(CAST(1 as int2))", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: i16 = row.get(0);

        assert_eq!(col, 2);
    });
}

#[test]
fn test_add_big_one() {
    test_in_db("adding", |conn| {
        let result = conn
            .query("SELECT add_big_one(CAST(1 as int8))", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: i64 = row.get(0);

        assert_eq!(col, 2);
    });
}

#[test]
fn test_add_together() {
    test_in_db("adding", |conn| {
        let result = conn
            .query(
                "SELECT add_together(CAST(1 as int8), CAST(2 as int4), CAST(3 as int2))",
                &[],
            )
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0);
        let col: i64 = row.get(0);

        assert_eq!(col, 6);
    });
}
