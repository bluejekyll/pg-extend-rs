extern crate cargo;
extern crate postgres;
extern crate tempfile;

use std::env;
use std::panic::{self, UnwindSafe};
use std::path::{Path, PathBuf};
use std::process;

use cargo::core::compiler::{Compilation, CompileMode};
use cargo::util::errors::CargoResult;
use postgres::Connection;

pub fn build_lib(name: &str) -> CargoResult<PathBuf> {
    println!("building library: {}", name);
    let cfg = cargo::util::config::Config::default()?;

    let mut opts = cargo::ops::CompileOptions::new(&cfg, CompileMode::Build)
        .expect("failed to get compile options");

    opts.spec = cargo::ops::Packages::Packages(vec![name.into()]);
    opts.filter = cargo::ops::CompileFilter::from_raw_arguments(
        true,
        vec![],
        false,
        vec![],
        false,
        vec![],
        false,
        vec![],
        false,
        false,
    );

    let search_path = if cfg.cwd().ends_with("integration-tests") {
        // if it's in the integration-tests, this is being run in the pg-extend-rs project
        cfg.cwd().parent().unwrap()
    } else {
        // otherwise we're in a different project
        cfg.cwd()
    };

    let path = cargo::util::important_paths::find_root_manifest_for_wd(search_path)?;
    println!("Cargo.toml: {}", path.display());
    let ws = cargo::core::Workspace::new(&path, &cfg)?;
    let result = cargo::ops::compile(&ws, &opts)?;
    Ok(get_lib_path(&result, name))
}

pub fn build_bin(name: &str) -> CargoResult<PathBuf> {
    println!("building binary: {}", name);
    let cfg = cargo::util::config::Config::default()?;

    let mut opts = cargo::ops::CompileOptions::new(&cfg, CompileMode::Build)
        .expect("failed to get compile options");
    opts.spec = cargo::ops::Packages::Packages(vec![name.into()]);
    opts.filter = cargo::ops::CompileFilter::from_raw_arguments(
        false,
        vec![],
        true,
        vec![],
        false,
        vec![],
        false,
        vec![],
        false,
        false,
    );

    let search_path = if cfg.cwd().ends_with("integration-tests") {
        // if it's in the integration-tests, this is being run in the pg-extend-rs project
        cfg.cwd().parent().unwrap()
    } else {
        // otherwise we're in a different project
        cfg.cwd()
    };

    let path = cargo::util::important_paths::find_root_manifest_for_wd(search_path)?;
    println!("Cargo.toml: {}", path.display());
    let ws = cargo::core::Workspace::new(&path, &cfg)?;
    let result = cargo::ops::compile(&ws, &opts)?;
    Ok(get_stmt_bin_path(&result))
}

fn get_lib_path(result: &Compilation, name: &str) -> PathBuf {
    let mut path = result.root_output.clone();
    path.push("debug");
    path.set_file_name(format!("lib{}", name));
    path.set_extension(if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    });
    path
}

fn get_stmt_bin_path(result: &Compilation) -> PathBuf {
    assert_eq!(1, result.binaries.len());
    result.binaries[0].clone()
}

pub fn db_conn() -> Connection {
    if let Ok(url) = env::var("POSTGRES_URL") {
        return Connection::connect(url, postgres::TlsMode::None).expect("could not connect");
    }

    let db_name = env::var("POSTGRES_TEST_DB").expect(
        "As a precaution, POSTGRES_TEST_DB must be set to ensure that other DBs are not damaged",
    );

    let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
    let user =
        env::var("POSTGRES_USER").unwrap_or_else(|_| env::var("USER").expect("USER is unset"));
    let conn_str = format!("postgres://{}@{}:{}/{}", user, host, port, db_name);

    Connection::connect(&conn_str as &str, postgres::TlsMode::None).expect("could not connect")
}

pub fn run_create_stmts(bin_path: &PathBuf, lib_path: &PathBuf) {
    let sql = process::Command::new(bin_path)
        .arg(lib_path)
        .output()
        .expect("failed to run get stmts");

    if !sql.status.success() {
        panic!(
            "get sql stmts failed: {}",
            String::from_utf8_lossy(&sql.stderr)
        );
    }

    let sql = String::from_utf8_lossy(&sql.stdout);
    let conn = db_conn();
    println!("executing stmts: {}", sql);
    conn.batch_execute(&sql).expect("failed to create function");
}

pub fn copy_to_tempdir(path: &Path, lib_path: PathBuf) -> PathBuf {
    let tmplib = path.with_file_name(lib_path.file_name().unwrap());
    assert!(
        path.exists(),
        format!("path does not exist: {}", path.display())
    );

    std::fs::copy(&lib_path, &tmplib)
        .map_err(|e| {
            format!(
                "failed to copy file from {} to {}: {}",
                lib_path.display(),
                tmplib.display(),
                e
            )
        })
        .unwrap();
    tmplib
}

pub fn test_in_db<F: FnOnce(Connection) + UnwindSafe>(lib_name: &str, test: F) {
    println!("test_in_db: {}", lib_name);
    let bin_path = build_bin(lib_name).expect("failed to build stmt binary");
    assert!(bin_path.exists());

    let lib_path = build_lib(lib_name).expect("failed to build extension");
    assert!(lib_path.exists());
    let tmpdir = tempfile::tempdir().expect("failed to make tempdir");
    let lib_path = copy_to_tempdir(tmpdir.path(), lib_path);

    println!("creating statements with bin: {}", bin_path.display());
    run_create_stmts(&bin_path, &lib_path);

    let panic_result = panic::catch_unwind(|| {
        let conn = db_conn();
        println!("executing on connection: {:?}", conn);
        test(conn)
    });

    // TODO: cleanup

    if let Err(e) = panic_result {
        panic::resume_unwind(e);
    }
}
