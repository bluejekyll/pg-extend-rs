// Copyright 2018-2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#[cfg(not(feature = "postgres-12"))]
#[cfg(feature = "fdw")]
extern crate pg_extend;

#[cfg(not(feature = "postgres-12"))]
#[cfg(feature = "fdw")]
use pg_extend::pg_create_stmt_bin;

#[cfg(not(feature = "postgres-12"))]
#[cfg(feature = "fdw")]
pg_create_stmt_bin!(DefaultFDW_pg_create_stmt);

#[cfg(any(feature = "postgres-12", not(feature = "fdw")))]
fn main() {
    println!("feature fdw must be enabled (also postgres 12 not currently supported)")
}
