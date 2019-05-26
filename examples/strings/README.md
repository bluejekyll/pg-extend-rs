# Rust based postgres extension

A strings example.

To build, get Rust, then (the RUSTFLAGS is required to build the library):

```console
$> RUSTFLAGS="-C link-arg=-undefineddynamic_lookup" cargo build --release
...
```

then load into postgres

```console
$> psql $CONN_STR
postgres=# CREATE FUNCTION concat_rs(text, text) RETURNS text AS 'path/to/crate/target/release/libstrings.dylib', 'strings' LANGUAGE C STRICT;
```