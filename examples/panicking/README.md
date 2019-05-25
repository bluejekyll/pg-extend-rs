# Rust based postgres extension

A panicking example.

To build, get Rust, then:

```console
$> cargo build --release
...
```

then load into postgres

```console
$> psql $CONN_STR
postgres=# CREATE FUNCTION panicking(integer) RETURNS integer AS 'path/to/crate/target/release/libpanicking.dylib', 'panicking' LANGUAGE C STRICT;
```