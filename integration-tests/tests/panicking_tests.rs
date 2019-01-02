extern crate integration_tests;

use integration_tests::*;

#[ignore]
#[test]
fn test_panicking() {
    test_in_db("panicking", |conn| {
        let result = conn.query("SELECT panicking(1)", &[]);
        assert!(result.is_err());
    });
}
