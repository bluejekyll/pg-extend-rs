# Rust based postgres extension

to build, get Rust, then:

```console
$> cargo build --release
...
```

then load into postgres

```console
$> psql $CONN_STR
postgres=# CREATE FUNCTION add_one(integer) RETURNS integer
 AS 'path/to/crate/target/release/lib_extension_name.dylib', 'add_one' LANGUAGE C STRICT;
```
 

