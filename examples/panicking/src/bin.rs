// becuase the lib is a cdylib... maybe there's a better way?
#[cfg(not(feature = "pg_allocator"))]
mod lib;

#[cfg(not(feature = "pg_allocator"))]
fn main() {
    const LIB_NAME: &'static str = env!("CARGO_PKG_NAME");

    println!("{}", lib::panicking_pg_create_stmt(&format!("target/release/lib{}.dylib", LIB_NAME)));
}

#[cfg(feature = "pg_allocator")]
fn main() {
    println!("disable `pg_allocator` feature to print create STMTs")
}