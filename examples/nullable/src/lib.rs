// Copyright 2019 Marti Raudsepp <marti@juffo.org>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate pg_extend;
extern crate pg_extern_attr;

use pg_extend::pg_magic;
use pg_extern_attr::pg_extern;

// This tells Postgres this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

/// Simply returns NULL. For testing a function with no arguments.
#[pg_extern]
fn get_null() -> Option<i32> {
    None
}

/// The NULLIF function returns a null value if value1 equals value2; otherwise it returns value1
/// https://www.postgresql.org/docs/current/functions-conditional.html#FUNCTIONS-NULLIF
#[pg_extern]
fn rs_nullif(value1: Option<String>, value2: Option<String>) -> Option<String> {
    if value1 == value2 {
        None
    } else {
        value1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_null() {
        assert_eq!(get_null(), None);
    }

    #[test]
    fn test_rs_nullif() {
        assert_eq!(
            rs_nullif(Some("a".to_string()), Some("-".to_string())),
            Some("a".to_string())
        );
        assert_eq!(
            rs_nullif(Some("a".to_string()), None),
            Some("a".to_string())
        );
        assert_eq!(
            rs_nullif(Some("-".to_string()), Some("-".to_string())),
            None
        );
        assert_eq!(rs_nullif(None, Some("-".to_string())), None);
        assert_eq!(rs_nullif(None, None), None);
    }
}
