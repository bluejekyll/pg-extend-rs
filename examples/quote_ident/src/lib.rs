// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate pg_extend;
extern crate pg_extern_attr;

use pg_extend::guard_pg;
use pg_extend::{pg_magic, pg_sys};
use pg_extern_attr::pg_extern;

// This tells Postgres this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

/// The pg_extern attribute wraps the function in the proper functions syntax for C extensions
#[pg_extern]
pub fn rust_quote_ident(value: String) -> String {
    use std::ffi::{CStr, CString};

    unsafe {
        guard_pg(|| {
            CStr::from_ptr(pg_sys::quote_identifier(
                CString::new(value.as_str()).unwrap().as_ptr(),
            ))
            .to_str()
            .unwrap()
            .to_owned()
        })
    }
}

#[cfg(test)]
mod tests {
    /* testing here would require a db connection */
}
