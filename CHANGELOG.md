# Change Log: pg-extend-rs

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## 0.2.0

### Added

- `PgAllocator` for allocating/deallocating through the Postgres `palloc` and `pfree` method.

## 0.1.0

### Added

- created `pg-extend` and `pg-extern-attr`
- `pg_extend::pg_sys` for postgres bindings
- `pg_extend::pg_datum` for conversions between Rust types and Postgres Datums
- `#[pg_extern]` that externalizes Rust functions for Postgres extensions
- panic handler to map panics to `FATAL` errors in Postgres
