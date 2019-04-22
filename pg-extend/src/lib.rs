// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Postgres extension library for Rust.

#![warn(missing_docs)]

pub mod pg_alloc;
#[macro_use]
pub mod pg_bool;
pub mod pg_datum;
pub mod pg_error;
pub mod pg_sys;
pub mod pg_type;
#[cfg(not(feature = "postgres-9"))]
pub mod pg_fdw;

/// A macro for marking a library compatible with the Postgres extension framework.
///
/// This macro was initially inspired from the `pg_module` macro in https://github.com/thehydroimpulse/postgres-extension.rs
#[macro_export]
macro_rules! pg_magic {
    (version: $vers:expr) => {
        // Set the global allocator to use postgres' allocator, which guarantees all memory freed at
        //   transaction close.
        #[global_allocator]
        #[cfg(feature = "pg_allocator")]
        static GLOBAL: pg_extend::pg_alloc::PgAllocator = pg_extend::pg_alloc::PgAllocator;

        #[no_mangle]
        #[allow(non_snake_case)]
        #[allow(unused)]
        #[link_name = "Pg_magic_func"]
        pub extern "C" fn Pg_magic_func() -> &'static pg_extend::pg_sys::Pg_magic_struct {
            use pg_extend::{pg_sys, register_panic_handler};
            use std::mem::size_of;
            use std::os::raw::c_int;

            const my_magic: pg_extend::pg_sys::Pg_magic_struct = pg_sys::Pg_magic_struct {
                len: size_of::<pg_sys::Pg_magic_struct>() as c_int,
                version: $vers as std::os::raw::c_int / 100,
                funcmaxargs: pg_sys::FUNC_MAX_ARGS as c_int,
                indexmaxkeys: pg_sys::INDEX_MAX_KEYS as c_int,
                namedatalen: pg_sys::NAMEDATALEN as c_int,
                float4byval: pg_sys::USE_FLOAT4_BYVAL as c_int,
                float8byval: pg_sys::USE_FLOAT8_BYVAL as c_int,
            };

            // TODO: is this a good idea here?
            // register panic_handler
            register_panic_handler();

            // return the magic
            &my_magic
        }
    };
}

/// Returns the slice of Datums, and a parallel slice which specifies if the Datum passed in is (SQL) NULL
pub fn get_args<'a>(
    func_call_info: &'a pg_sys::FunctionCallInfoData,
) -> (impl 'a + Iterator<Item=&pg_sys::Datum>, impl 'a + Iterator<Item=pg_bool::Bool>) {
    let num_args = func_call_info.nargs as usize;

    let args = func_call_info.arg[..num_args].iter();
    let args_null =
        func_call_info.argnull[..num_args]
        .iter()
        .map(|b| pg_bool::Bool::from(*b));

    (args, args_null)
}

/// This will replace the current panic_handler
pub fn register_panic_handler() {
    use std::panic;
    use crate::pg_error;

    // set (and replace the existing) panic handler, this will tell Postgres that the call failed
    //   a level of Fatal will force the DB connection to be killed.
    panic::set_hook(Box::new(|info| {
        let level = pg_error::Level::Fatal;

        pg_error::log(level, file!(), line!(), module_path!(), format!("panic in Rust extension: {}", info));
    }));
}

/// auto generate function to output a SQL create statement for the function
/// 
/// Until concat_ident! stabilizes, this requires the name to passed with the appended sctring
///   `_pg_create_stmt`
/// 
/// # Example
/// 
/// create a binary for the library, like bin.rs, and this will generate a `main()` function in it
/// 
/// ```text
/// extern crate pg_extend;
///
/// use pg_extend::pg_create_stmt_bin;
///
/// pg_create_stmt_bin!(
///     add_one_pg_create_stmt,
///     add_big_one_pg_create_stmt,
///     add_small_one_pg_create_stmt,
///     add_together_pg_create_stmt
/// );
/// ```
#[macro_export]
macro_rules! pg_create_stmt_bin {
    ( $( $func:ident ),* ) => {
        use std::env;

        // becuase the lib is a cdylib... maybe there's a better way?
        #[cfg(not(feature = "pg_allocator"))]
        mod lib;

        #[cfg(target_os = "linux")]
        const DYLIB_EXT: &str = "so";

        #[cfg(target_os = "macos")]
        const DYLIB_EXT: &str = "dylib";

        #[cfg(not(feature = "pg_allocator"))]
        fn main() {
            const LIB_NAME: &str = env!("CARGO_PKG_NAME");

            let lib_path = env::args().nth(1).unwrap_or_else(|| format!("target/release/lib{}.{}", LIB_NAME, DYLIB_EXT));
            
            $( println!("{}", lib::$func(&lib_path)); )*
        }

        #[cfg(feature = "pg_allocator")]
        fn main() {
            panic!("disable `pg_allocator` feature to print create STMTs")
        }
    };
}
