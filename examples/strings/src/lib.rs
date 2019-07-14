// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate pg_extend;
extern crate pg_extern_attr;

use pg_extend::native::Text;
use pg_extend::pg_alloc::PgAllocator;
use pg_extend::pg_magic;
use pg_extern_attr::pg_extern;
use pg_extend::info;

// This tells Postges this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

/// The pg_extern attribute wraps the function in the proper functions syntax for C extensions
#[pg_extern]
fn concat_rs(mut a: String, b: String) -> String {
    a.push_str(&b);

    a
}

/// Zero overhead Text types directly from PG, this requires the PgAllocator for the associated lifetime.
#[pg_extern]
fn text_rs<'mc>(_alloc: &'mc PgAllocator, text: Text<'mc>) -> Text<'mc> {
    info!("Length of text: {}", text.len());
    
    // deref to a &str
    let rust_str: &str = &text;

    info!("Length of rust_str: {}", rust_str.len());
    info!("Text as str from: {}", rust_str);
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_rs() {
        assert_eq!(&concat_rs("a".to_string(), "b".to_string()), "ab");
    }
}
