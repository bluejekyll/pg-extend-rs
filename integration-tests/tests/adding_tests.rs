extern crate integration_tests;

use integration_tests::*;

#[test]
fn test_add_one() {
    test_in_db("adding", |mut conn| {
        let result = conn.query("SELECT add_one(1)", &[]).expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: i32 = row.get(0);

        assert_eq!(col, 2);

        // Calling the function with NULL argument returns NULL because it's declared STRICT
        let result = conn
            .query("SELECT add_one(NULL)", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: Option<i32> = row.get(0);

        assert_eq!(col, None);
    });
}

#[test]
fn test_add_one_null() {
    test_in_db("adding", |mut conn| {
        // Rust add_big_one function should not be called because we declare it STRICT.
        let result = conn
            .query("SELECT add_big_one(CAST(NULL as int8))", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: Option<i64> = row.get(0);

        assert_eq!(col, None);
    });
}

#[test]
fn test_add_small_one() {
    test_in_db("adding", |mut conn| {
        let result = conn
            .query("SELECT add_small_one(CAST(1 as int2))", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: i16 = row.get(0);

        assert_eq!(col, 2);
    });
}

#[test]
fn test_add_big_one() {
    test_in_db("adding", |mut conn| {
        let result = conn
            .query("SELECT add_big_one(CAST(1 as int8))", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: i64 = row.get(0);

        assert_eq!(col, 2);
    });
}

#[test]
fn test_add_together() {
    test_in_db("adding", |mut conn| {
        let result = conn
            .query(
                "SELECT add_together(CAST(1 as int8), CAST(2 as int4), CAST(3 as int2))",
                &[],
            )
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: i64 = row.get(0);

        assert_eq!(col, 6);
    });
}

#[test]
fn test_sum_array() {
    test_in_db("adding", |mut conn| {
        let result = conn
            .query("SELECT sum_array(ARRAY[1, 2, 3])", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: i32 = row.get(0);

        assert_eq!(col, 6);
    });
}

#[test]
fn test_sum_small_array() {
    test_in_db("adding", |mut conn| {
        let result = conn
            .query("SELECT sum_small_array(ARRAY[1, 2, 3]::int2[])", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: i16 = row.get(0);

        assert_eq!(col, 6);
    });
}

#[test]
fn test_sum_big_array() {
    test_in_db("adding", |mut conn| {
        let result = conn
            .query("SELECT sum_big_array(ARRAY[1, 2, 3]::int8[])", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: i64 = row.get(0);

        assert_eq!(col, 6);
    });
}

#[test]
fn test_sum_float_array() {
    test_in_db("adding", |mut conn| {
        let result = conn
            .query("SELECT sum_float_array(ARRAY[1.1, 2.2, 3.3])", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: f32 = row.get(0);

        assert_eq!(format!("{:.1}", col), "6.6".to_owned());
    });
}

#[test]
fn test_sum_double_array() {
    test_in_db("adding", |mut conn| {
        let result = conn
            .query("SELECT sum_double_array(ARRAY[1.1, 2.2, 3.3])", &[])
            .expect("query failed");
        assert_eq!(result.len(), 1);

        let row = result.get(0).expect("no rows returned");
        let col: f64 = row.get(0);

        assert_eq!(format!("{:.1}", col), "6.6".to_owned());
    });
}
