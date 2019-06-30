extern crate integration_tests;

use integration_tests::*;

#[test]
#[ignore]
fn test_fdw() {
    test_in_db("fdw", |conn| {
        conn.batch_execute(
            "
DROP SERVER IF EXISTS df CASCADE;
CREATE SERVER df FOREIGN DATA WRAPPER defaultfdw;

DROP SCHEMA IF EXISTS fdw_test_schema CASCADE;
CREATE SCHEMA fdw_test_schema;

IMPORT FOREIGN SCHEMA test
  FROM SERVER df
  INTO fdw_test_schema;
",
        )
        .expect("Failed to import foreign schema");

        let rows = conn
            .query("SELECT * FROM fdw_test_schema.mytable;", &[])
            .expect("Failed to query FDW");
        assert_eq!(rows.len(), 5);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(row.len(), 1);
            assert_eq!((i + 1) as i32, row.get::<_, i32>(0))
        }
    })
}
