// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate bindgen;

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("postgres.rs");

    if !out_path.exists() {
        let pg_include = env::var("PG_INCLUDE_PATH")
            .expect("set environment variable PG_INCLUDE_PATH to the Postgres install include dir, e.g. /var/lib/pgsql/include/server");

        // println!("cargo:rustc-link-search=/Users/benjaminfry/Downloads/postgresql-11.1/src/common");
        // println!("cargo:rustc-link-lib=pgcommon_srv");

        // pkg_config::Config::new().atleast_version("11.1").probe("libpq").unwrap();
        // pkg_config::Config::new().atleast_version("11.1").probe("libecpg").unwrap();

        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.

        // these cause duplicate definition problems on linux
        // see: https://github.com/rust-lang/rust-bindgen/issues/687
        let ignored_macros = IgnoreMacros(
            vec![
                "FP_INFINITE".into(),
                "FP_NAN".into(),
                "FP_NORMAL".into(),
                "FP_SUBNORMAL".into(),
                "FP_ZERO".into(),
                "IPPORT_RESERVED".into(),
            ]
            .into_iter()
            .collect(),
        );

        let bindings = bindgen::Builder::default()
            .clang_arg(format!("-I{}", pg_include))
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.h")
            .parse_callbacks(Box::new(ignored_macros))
            .rustfmt_bindings(true)
            // FIXME: add this back
            .layout_tests(false);

        // Finish the builder and generate the bindings.
        let bindings = bindings
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/postgres.rs file.
        bindings
            .write_to_file(out_path)
            .expect("Couldn't write bindings!");
    }
}

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}
