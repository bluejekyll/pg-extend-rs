// TODO: build a macro for this

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
    const LIB_NAME: &'static str = env!("CARGO_PKG_NAME");

    let lib_path = env::args().skip(1).next().unwrap_or_else(|| format!("target/release/lib{}.{}", LIB_NAME, DYLIB_EXT));

    println!("{}", lib::add_one_pg_create_stmt(&lib_path));
    println!("{}", lib::add_small_one_pg_create_stmt(&lib_path));
    println!("{}", lib::add_big_one_pg_create_stmt(&lib_path));
    println!("{}", lib::add_together_pg_create_stmt(&lib_path));
}

#[cfg(feature = "pg_allocator")]
fn main() {
    panic!("disable `pg_allocator` feature to print create STMTs");
}