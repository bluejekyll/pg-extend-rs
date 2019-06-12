# Example Postgres extension using strings

To build, get Rust, then:

```console
$> cargo build --release
...
```

then load into Postgres:

```console
$> psql $CONN_STR
postgres=# CREATE FUNCTION concat_rs(text, text) RETURNS text AS 'path/to/crate/target/release/libstrings.dylib', 'pg_concat_rs' LANGUAGE C STRICT;
```
