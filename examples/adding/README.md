# Rust based postgres extension

An example of adding 1 to another number and returning the result.

To build, get Rust, then:

```console
$> cargo build --release
...
```

then load into postgres

```console
$> psql $CONN_STR
postgres=# CREATE FUNCTION add_one(integer) RETURNS integer AS 'path/to/crate/target/release/libadding.dylib', 'pg_add_one' LANGUAGE C STRICT;
```