extern crate integration_tests;

use core::mem;
use std::sync::{Arc, Mutex};

use postgres::error::DbError;
use postgres::Client;

use integration_tests::*;

#[test]
fn test_rs_error() {
    test_in_db("logging", |mut conn| {
        let result = conn.query("SELECT rs_error('No you dont!')", &[]);
        assert!(result.is_err());

        if let Err(err) = result {
            assert_eq!(format!("{}", err), "db error: ERROR: No you dont!");
        } else {
            panic!("should have been an error");
        }
    });
}

struct MsgCapture {
    msgs: Arc<Mutex<Vec<DbError>>>,
}

/// Allows capturing log messages from a PostgreSQL connection.
impl MsgCapture {
    fn new() -> MsgCapture {
        MsgCapture {
            msgs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    /// Returns the current message buffer and flushes it.
    fn drain(&self) -> Vec<DbError> {
        let mut msgs = self.msgs.lock().unwrap();
        mem::replace(&mut *msgs, Vec::new())
    }
}

#[test]
#[ignore] // the new Postgres connection impl made it not possible to capture the logging output
fn test_rs_log_all() {
    test_in_db("logging", |mut conn: Client| {
        let capture = MsgCapture::new();

        // Test with log level ERROR
        // INFO messages are sent ot the client even at log level ERROR
        // https://www.postgresql.org/docs/current/runtime-config-client.html#GUC-CLIENT-MIN-MESSAGES
        conn.query("SET client_min_messages=error", &[])
            .expect("query failed");
        //        let old_handler = conn.set_notice_handler(capture.get_handler());

        conn.query("SELECT rs_log_all()", &[])
            .expect("query failed");

        let msgs = capture.drain();
        assert_eq!(msgs[0].severity(), "INFO");
        assert_eq!(msgs[0].message(), "TEST: This is an info message");
        assert_eq!(msgs.len(), 1);

        // Test with log level DEBUG5
        conn.query("SET client_min_messages=debug5", &[])
            .expect("query failed");
        conn.query("SELECT rs_log_all()", &[])
            .expect("query failed");

        // Filter out PostgreSQL's own debug messages e.g.: "DEBUG: StartTransaction(1) ..."
        // Our test messages all start with "TEST: "
        let msgs: Vec<String> = capture
            .drain()
            .iter()
            .filter_map(|m: &DbError| {
                if m.severity() != "DEBUG" || m.message().starts_with("TEST: ") {
                    Some(format!("{}: {}", m.severity(), m.message()))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            msgs,
            vec![
                "WARNING: TEST: This is a warning",
                "NOTICE: TEST: Notice this!",
                "INFO: TEST: This is an info message",
                "LOG: TEST: This is an LOG-level message",
                // PostgreSQL clients don't distinguish between DEBUG1...DEBUG5 levels.
                "DEBUG: TEST: This is a debug message",
                "DEBUG: TEST: This is a trace-level message"
            ]
        );

        // Clean up, restore old notice handler.
        conn.query("RESET client_min_messages", &[])
            .expect("query failed");
        //      conn.set_notice_handler(old_handler);
    });
}
