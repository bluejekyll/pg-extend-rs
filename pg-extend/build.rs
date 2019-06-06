// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate bindgen;
extern crate clang;

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("postgres.rs");

    // Re-run this if wrapper.h changes
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=PG_INCLUDE_PATH");

    let pg_include = include_dir()
        .expect("set environment variable PG_INCLUDE_PATH to the Postgres install include dir, e.g. /var/lib/pgsql/include/server");

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

    let feature_version = get_postgres_feature_version(pg_include);
    println!("cargo:rustc-cfg=feature=\"{}\"", feature_version)
}

fn include_dir() -> Result<String, env::VarError> {
    env::var("PG_INCLUDE_PATH").or_else(|err| {
        match Command::new("pg_config").arg("--includedir-server").output() {
            Ok(out) => Ok(String::from_utf8(out.stdout).unwrap().trim().to_string()),
            Err(..) => Err(err),
        }
    })
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

fn get_postgres_feature_version(pg_include: String) -> &'static str {
    let clang = clang::Clang::new().unwrap();
    let index = clang::Index::new(&clang, false, false);
    let repr = index.parser("pg_majorversion.h")
        .arguments(&[format!("-I{}", pg_include)])
        .parse()
        .expect("failed to parse pg_config.h");

    // Find the variable declaration
    let major_version = repr.get_entity().get_children().into_iter().find(|e| {
        e.get_kind() == clang::EntityKind::VarDecl
            && e.get_name() == Some("pg_majorversion".into())
    }).expect("Couldn't find major version");

    // Find the string literal within the declaration
    let string_literal = major_version.get_children().into_iter().find(|e| {
        e.get_kind() == clang::EntityKind::StringLiteral
    }).expect("couldn't find string literal for major version");

    let version = string_literal.get_display_name().unwrap().replace("\"", "");
    let version = version.split(".").collect::<Vec<_>>();

    match &version[..] {
        ["9", _] => "postgres-9",
        ["10"] => "postgres-10",
        ["11"] => "postgres-11",
        val => panic!("unknown Postgres version {:?}", val),
    }
}
