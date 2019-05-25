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

/// This tells Postges this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

/// The pg_extern attribute wraps the function in the proper functions syntax for C extensions
#[pg_extern]
fn threads() -> String {
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threads() {
        
    }
}
