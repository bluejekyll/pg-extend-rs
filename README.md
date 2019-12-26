
[![Build Status](https://github.com/bluejekyll/pg-extend-rs/workflows/test/badge.svg?branch=master)](https://github.com/bluejekyll/pg-extend-rs/actions?query=workflow%3Atest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](LICENSE-APACHE)
[![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=bluejekyll/pg-extend-rs)](https://dependabot.com)
[![](http://meritbadge.herokuapp.com/pg-extend)](https://crates.io/crates/pg-extend)
[![Discord](https://img.shields.io/discord/589988605322199149.svg)](https://discord.gg/y7ZvY5p)

# Rust based Postgres extension

The main things provided by this crate are some macros that help with writing Postgres extensions in Rust.

The objective (not all these are yet implemented):

- Automatic type conversions, see `PgDatum` and `TryFromPgDatum` to `Into<PgDatum>`
- `pg_magic` macro for declaring libraries as Postgres extensions
- `pg_extern` attribute for wrapping Rust functions in Postgres C style definitions
- panic handlers for conversion into Postgres errors
- allocator that uses Postgres `palloc` allocator and `pfree`
- *tbd* integrate postgres error logs with `log`
- *tbd* support all Datum types
- *tbd* support table like returns and manipulation
- *tbd* generators for the psql scripts to load functions

## Getting started

This project uses `cargo-make` for automation. While not necessary, it does help with a lot of the build tasks, so is recommended. This can be installed with `cargo install cargo-make`.

Once installed, it will install Postgres into the `target` directory for testing. There are profiles for each supported Postgres version, `v10`, `v11`, and `v12`. The specific minor version used is in 

To run all tests with all features, for example, run:

```shell
> cargo make all-features -p v12 # if -p is left off, then the default is v12
```

## Building

If using `cargo-make` then the environment variable `PG_DIR` can be used to specify the location of the Postgres install.

First install Postgres. The build should be able to find the directory for the Postgres server headers, it uses the `pg_config --includedir-server` to attempt to find the directory. If it is unsuccessful then this environment variable is required:

`PG_INCLUDE_PATH=[/path/to/postgres]/include/server # e.g. /usr/local/pgsql/include/server`

For the dynamic library to compile, your project should also have `.cargo/config` file with content:

```toml
[target.'cfg(unix)']
rustflags = "-C link-arg=-undefineddynamic_lookup"

[target.'cfg(windows)']
rustflags = "-C link-arg=/FORCE"
```

This informs the linker that some of the symbols for Postgres won't be available until runtime on the dynamic library load.

## Running the integration tests

Standard tests can be run with the normal `cargo test`, but the integration tests are a little more involved. They require a connection to an actual Postgres DB. These instructions were performed on macOS. Create a DB in Postgres to be use. In this example a DB was created in the `/usr/local/var/posgres` path, with the name `postgres`. When using `cargo-make` all the automation of starting, installing and setting up the DB is handled for you:

Test all features:

```shell
> cargo make all-features
```

Test default features:

```shell
> cargo make default-features
```

Test no-default-features:

```shell
> cargo make no-default-features
```

Testing against different versions; `v10`, `v11`, `v12` are valid:

```shell
> cargo make all-features -p v10
```

### Without cargo-make

To run the test must know the DB name to use, the DB must be running, and then the tests can be run:

```shell
export POSTGRES_TEST_DB=postgres

pg_ctl -D /usr/local/var/postgres start
cargo test
```

## Examples

- [adding](https://github.com/bluejekyll/pg-extend-rs/tree/master/examples/adding)
- [panicking](https://github.com/bluejekyll/pg-extend-rs/tree/master/examples/panicking)

## Features

TBD

## Community

For live discussions beyond this repository, please see this [Discord](https://discord.gg/y7ZvY5p).
