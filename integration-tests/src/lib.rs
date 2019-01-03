extern crate postgres;

use std::env;
use std::panic::{self, UnwindSafe};
use std::path::PathBuf;
use std::process;

use postgres::Connection;

#[cfg(target_os = "linux")]
const DYLIB_EXT: &str = "so";

#[cfg(target_os = "macos")]
const DYLIB_EXT: &str = "dylib";

const LIB_DIR: &str = "target/integration-libs";
const BIN_DIR: &str = "target/integration-bins";

pub fn lib_path(name: &str) -> PathBuf {
    let working_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is unset");
    let mut path = PathBuf::new();

    path.push(working_dir);
    path.push(LIB_DIR);
    path.push("debug");
    path.push(format!("lib{}.{}", name, DYLIB_EXT));

    path
}

pub fn build_sql_lib(name: &str) {
    let cargo = env::var("CARGO").expect("CARGO bin env var not set");

    let status = process::Command::new(cargo)
        .env("RUSTFLAGS", "-C link-arg=-undefineddynamic_lookup")
        .arg("build")
        .arg(format!("--manifest-path=../examples/{}/Cargo.toml", name))
        .arg(format!("--target-dir={}", LIB_DIR))
        .arg("--lib")
        .arg("--features=pg_allocator")
        .status()
        .expect("failed to run build --lib");

    #[cfg(target_family = "unix")]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let lib_path = lib_path(name);
        let metadata = fs::metadata(&lib_path).expect("could not get metadata for library");
        let mut permissions = metadata.permissions();

        // making sure the file is readable and executable by the world
        println!(
            "making {} world readable and executable",
            lib_path.display()
        );
        permissions.set_mode(0o755);

        let lib_dir = lib_path.parent().unwrap();
        let metadata = fs::metadata(&lib_dir).expect("could not get metadata for library dir");
        let mut permissions = metadata.permissions();

        println!("making {} world readable and executable", lib_dir.display());
        permissions.set_mode(0o755);
    }

    assert!(status.success(), "build --lib failed");
}

pub fn build_sql_create_stmt(name: &str) {
    let cargo = env::var("CARGO").expect("CARGO bin env var not set");
    let bin_name = format!("{}-stmt", name);

    let status = process::Command::new(cargo)
        .arg("build")
        .arg(format!("--manifest-path=../examples/{}/Cargo.toml", name))
        .arg(format!("--target-dir={}", BIN_DIR))
        .arg(format!("--bin={}", bin_name))
        .status()
        .expect("failed to run build --bin");

    assert!(status.success(), "build --bin failed");
}

pub fn db_conn() -> Connection {
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

pub fn run_create_stmts(name: &str) {
    let lib_path = lib_path(name);
    let sql = process::Command::new(&format!("{}/debug/{}-stmt", BIN_DIR, name))
        .arg(lib_path.to_str().unwrap())
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

pub fn test_in_db<F: FnOnce(Connection) + UnwindSafe>(lib_name: &str, test: F) {
    build_sql_lib(lib_name);
    build_sql_create_stmt(lib_name);
    run_create_stmts(lib_name);

    let panic_result = panic::catch_unwind(|| {
        let conn = db_conn();
        test(conn)
    });

    // TODO: cleanup

    if let Err(e) = panic_result {
        panic::resume_unwind(e);
    }
}
