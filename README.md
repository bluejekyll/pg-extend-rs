# Rust based postgres extension

The main things provided by this crate are some macros that help with writing Postgres extensions in Rust.

The objective (not all these are yet implemented):

- Automatic type conversions, see `PgDatum` and `TryFromPgDatum` to `Into<PgDatum>`
- `pg_magic` macro for declaring libraries as Postgres extensions
- `pg_extern` attribute for wrapping Rust functions in Postgres C style definitions
- *tbd* allocator that uses Postgres allocators
- *tbd* panic handlers for conversion into Postgres errors
- *tbd* integrate postgres error logs with `log`

## Building

First install Postgres. Once installed, this environment variable is required:

`PG_INCLUDE_PATH=[/path/to/postgres]/include/server # e.g. /usr/local/pgsql/include/server`

This environment variable is also required for the dynamic libraries to compile:

`RUSTFLAGS="-C link-arg=-undefineddynamic_lookup"`

This informs the linker that some of the symbols for postgres won't be available until runtime on the dynamic library load.

## Examples

- [add_one](examples/add_one)