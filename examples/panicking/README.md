# Rust based postgres extension

A panicking example.

To build, get Rust, then (the RUSTFLAGS is required to build the library):

```console
$> RUSTFLAGS="-C link-arg=-undefineddynamic_lookup" cargo build --release
...
```

then load into postgres

```console
$> psql $CONN_STR
postgres=# CREATE FUNCTION panicking(integer) RETURNS integer AS 'path/to/crate/target/release/libpanicking.dylib', 'panicking' LANGUAGE C STRICT;
```