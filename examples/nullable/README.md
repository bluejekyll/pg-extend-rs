# Rust based postgres extension

An example of the NULLIF() conditional expression implemented in Rust, taking
advntage of optional (NULL or Option<>) arguments and return type.

To build, get Rust, then (the RUSTFLAGS is required to build the library):

```console
$> RUSTFLAGS="-C link-arg=-undefineddynamic_lookup" cargo build --release
...
```

then load into postgres

```console
$> psql $CONN_STR
postgres=# CREATE FUNCTION rs_nullif(text,text) RETURNS text AS 'path/to/libnullable.dylib', 'pg_rs_nullif' LANGUAGE C;
```
