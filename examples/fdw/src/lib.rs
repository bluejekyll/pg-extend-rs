// Copyright 2018 Liz Frost <web@stillinbeta.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate pg_extend;
extern crate pg_extern_attr;

use pg_extend::pg_fdw::ForeignWrapper;
use pg_extend::pg_magic;
use pg_extern_attr::pg_foreignwrapper;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

/// This tells Postges this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

#[pg_foreignwrapper]
struct DefaultFDW;

impl ForeignWrapper for DefaultFDW {}

