extern crate pg_extern_attr;

use pg_extern_attr::pg_extern;

/// This tells Postges this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

/// The pg_extern attribute wraps the function in the proper functions syntax for C extensions
#[pg_extern]
fn add_one(value: i32) -> i32 {
    (value + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_one() {
        assert_eq!(add_one(1), 2);
    }
}
