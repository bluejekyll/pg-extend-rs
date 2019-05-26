// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate pg_extern_attr;
extern crate pg_extend;

use pg_extern_attr::pg_extern;
use pg_extend::pg_magic;

// This tells Postgres this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);


/// The pg_extern attribute wraps the function in the proper functions syntax for C extensions
#[pg_extern]
fn add_one(value: i32) -> i32 {
    (value + 1)
}

/// Test the i16 value
#[pg_extern]
fn add_small_one(value: i16) -> i16 {
    (value + 1)
}

/// Test the i16 value
#[pg_extern]
fn add_big_one(value: i64) -> i64 {
    (value + 1)
}

/// Test the i16 value
#[pg_extern]
fn add_together(v1: i64, v2: i32, v3: i16) -> i64 {
    (v1 + i64::from(v2) + i64::from(v3))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_one() {
        assert_eq!(add_one(1), 2);
    }
}
