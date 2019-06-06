# Example Postgres extension using panic

Demonstrating how panic is caught by pg-extend-rs.

To build, get Rust, then:

```console
$> cargo build --release
...
```

then load into Postgres:

```console
$> psql $CONN_STR
postgres=# CREATE FUNCTION panicking(integer) RETURNS integer AS 'path/to/crate/target/release/libpanicking.dylib', 'panicking' LANGUAGE C STRICT;
```