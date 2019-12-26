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

    let bindings = get_bindings(&pg_include) // Gets initial bindings that are OS-dependant
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
    println!("cargo:rustc-cfg=feature=\"{}\"", feature_version);
}

#[cfg(windows)]
fn get_bindings(pg_include: &str) -> bindgen::Builder {
    // Compilation in windows requires these extra inclde paths
    let pg_include_win32_msvc = format!("{}\\port\\win32_msvc", pg_include);
    let pg_include_win32 = format!("{}\\port\\win32", pg_include);
    // The `pg_include` path comes in the format og "includes/server", but we also need
    // the parent folder, so we remove the "/server" part at the end
    let pg_include_parent = pg_include[..(pg_include.len() - 7)].to_owned();

    bindgen::Builder::default()
        .clang_arg(format!("-I{}", pg_include_win32_msvc))
        .clang_arg(format!("-I{}", pg_include_win32))
        .clang_arg(format!("-I{}", pg_include))
        .clang_arg(format!("-I{}", pg_include_parent))
        // Whitelist all PG-related functions
        .whitelist_function("pg.*")
        // Whitelist used functions
        .whitelist_function("longjmp")
        .whitelist_function("_setjmp")
        .whitelist_function("cstring_to_text")
        .whitelist_function("text_to_cstring")
        .whitelist_function("errmsg")
        .whitelist_function("errstart")
        .whitelist_function("errfinish")
        .whitelist_function("pfree")
        .whitelist_function("list_.*")
        // Whitelist all PG-related types
        .whitelist_type("PG.*")
        // Whitelist used types
        .whitelist_type("jmp_buf")
        .whitelist_type("text")
        .whitelist_type("varattrib_1b")
        .whitelist_type("varattrib_4b")
        // Whitelist PG-related values
        .whitelist_var("PG.*")
        // Whitelist log-level values
        .whitelist_var("DEBUG.*")
        .whitelist_var("LOG.*")
        .whitelist_var("INFO")
        .whitelist_var("NOTICE")
        .whitelist_var("WARNING")
        .whitelist_var("ERROR")
        .whitelist_var("FATAL")
        .whitelist_var("PANIC")
        // Whitelist misc values
        .whitelist_var("CurrentMemoryContext")
        .whitelist_var("FUNC_MAX_ARGS")
        .whitelist_var("INDEX_MAX_KEYS")
        .whitelist_var("NAMEDATALEN")
        .whitelist_var("USE_FLOAT.*")
        // FDW whitelisting
        .whitelist_function("pstrdup")
        .whitelist_function("lappend")
        .whitelist_function("makeTargetEntry")
        .whitelist_function("makeVar")
        .whitelist_function("ExecStoreTuple")
        .whitelist_function("heap_form_tuple")
        .whitelist_function("ExecClearTuple")
        .whitelist_function("slot_getallattrs")
        .whitelist_function("get_rel_name")
        .whitelist_function("GetForeignTable")
        .whitelist_function("GetForeignServer")
        .whitelist_function("make_foreignscan")
        .whitelist_function("extract_actual_clauses")
        .whitelist_function("add_path")
        .whitelist_function("create_foreignscan_path")
        .whitelist_type("ImportForeignSchemaStmt")
        .whitelist_type("ResultRelInfo")
        .whitelist_type("EState")
        .whitelist_type("ModifyTableState")
        .whitelist_type("Relation")
        .whitelist_type("RangeTblEntry")
        .whitelist_type("Query")
        .whitelist_type("ForeignScanState")
        .whitelist_type("InvalidBuffer")
        .whitelist_type("RelationData")
        .whitelist_type("ForeignScan")
        .whitelist_type("Plan")
        .whitelist_type("ForeignPath")
        .whitelist_type("RelOptInfo")
        .whitelist_type("Form_pg_attribute")
        .whitelist_type("DefElem")
        .whitelist_type("Value")
        .whitelist_var("InvalidBuffer")
}

#[cfg(unix)]
fn get_bindings(pg_include: &str) -> bindgen::Builder {
    bindgen::Builder::default().clang_arg(format!("-I{}", pg_include))
}

fn include_dir() -> Result<String, env::VarError> {
    env::var("PG_INCLUDE_PATH").or_else(|err| {
        match Command::new("pg_config")
            .arg("--includedir-server")
            .output()
        {
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
    let repr = index
        .parser("pg_majorversion.h")
        .arguments(&[format!("-I{}", pg_include)])
        .parse()
        .expect("failed to parse pg_config.h");

    // Find the variable declaration
    let major_version = repr
        .get_entity()
        .get_children()
        .into_iter()
        .find(|e| {
            e.get_kind() == clang::EntityKind::VarDecl
                && e.get_name() == Some("pg_majorversion".into())
        })
        .expect("Couldn't find major version");

    // Find the string literal within the declaration
    let string_literal = major_version
        .get_children()
        .into_iter()
        .find(|e| e.get_kind() == clang::EntityKind::StringLiteral)
        .expect("couldn't find string literal for major version");

    let version = string_literal.get_display_name().unwrap().replace("\"", "");
    let version = version.split(".").collect::<Vec<_>>();

    match &version[..] {
        ["9", _] => "postgres-9",
        ["10"] => "postgres-10",
        ["11"] => "postgres-11",
        ["12"] => "postgres-12",
        val => panic!("unknown Postgres version {:?}", val),
    }
}
