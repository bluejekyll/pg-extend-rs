extern crate integration_tests;

use integration_tests::*;

#[test]
fn test_panicking() {
    test_in_db("panicking", |conn| {
        let result = conn.query("SELECT panicking(1)", &[]);
        assert!(result.is_err());
    });
}

#[test]
fn test_longjmping() {
    test_in_db("panicking", |conn| {
        let result = conn.query("SELECT longjmping(3)", &[]);
        assert!(result.is_err());
    });
}

