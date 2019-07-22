// Copyright 2018 Liz Frost <web@stillinbeta.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate pg_extend;
extern crate pg_extern_attr;

use pg_extend::pg_fdw::{ForeignData, ForeignRow, OptionMap};
use pg_extend::{pg_datum, pg_magic, pg_type};
use pg_extern_attr::pg_foreignwrapper;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// This tells Postges this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

#[pg_foreignwrapper]
struct DefaultFDW {
    i: i32,
}

struct MyRow {
    i: i32,
}

impl ForeignRow for MyRow {
    fn get_field(
        &self,
        _name: &str,
        _typ: pg_type::PgType,
        _opts: OptionMap,
    ) -> Result<Option<pg_datum::PgDatum>, &str> {
        Ok(Some(self.i.into()))
    }
}

impl Iterator for DefaultFDW {
    type Item = Box<ForeignRow>;
    fn next(&mut self) -> Option<Self::Item> {
        self.i += 1;
        if self.i > 5 {
            None
        } else {
            Some(Box::new(MyRow { i: self.i }))
        }
    }
}

impl ForeignData for DefaultFDW {
    fn begin(_sopts: OptionMap, _topts: OptionMap, _table_name: String) -> Self {
        DefaultFDW { i: 0 }
    }

    fn schema(
        _server_opts: OptionMap,
        server_name: String,
        _remote_schema: String,
        local_schema: String,
    ) -> Option<Vec<String>> {
        Some(vec![format!(
            "CREATE FOREIGN TABLE {schema}.mytable (number Integer) SERVER {server}",
            server = server_name,
            schema = local_schema
        )])
    }
}
