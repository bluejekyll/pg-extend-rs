// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate pg_extend;
extern crate pg_extern_attr;

use pg_extend::pg_magic;
use pg_extern_attr::pg_extern;
// This tells Postges this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

/// The pg_extern attribute wraps the function in the proper functions syntax for C extensions
#[pg_extern]
fn panicking(value: i32) -> i32 {
    panic!("forced panic in Rust example, value: {}", value);
}

/// Tests a longjmp
///
/// Don't actually do this, it's a test for Postgres' usage of longjmp
#[pg_extern]
fn longjmping(value: i32) -> i32 {
    use pg_extend::error;

    error!("this error will longjmp: {}", value);

    unreachable!("IF YOU'RE SEEING THIS, LONGJMP FAILED");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_panicking() {
        assert_eq!(panicking(1), 2);
    }
}
