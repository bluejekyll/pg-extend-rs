extern crate cargo;
extern crate postgres;
extern crate users;
extern crate tempfile;

use cargo::util::errors::CargoResult;
use postgres::{Connection, TlsMode};
use std::path::PathBuf;

const SQL_FUNC_NAME: &str = "number_fdw";
const C_FUNC_NAME: &str = "fdw_DefaultFDW";

fn build_fdw() -> CargoResult<PathBuf> {
    let cfg = cargo::util::config::Config::default()?;
    let opts = cargo::ops::CompileOptions::new(&cfg, cargo::core::compiler::CompileMode::Build)?;
    let path = cargo::util::important_paths::find_root_manifest_for_wd(cfg.cwd())?;
    let ws = cargo::core::Workspace::new(&path, &cfg)?;
    let result = cargo::ops::compile(&ws, &opts)?;

    let mut path = result.root_output;
    path.push("debug");
    path.set_file_name("libfdw");
    path.set_extension(if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    });
    Ok(path)
}

fn get_url() -> String {
    if let Ok(v) = std::env::var("POSTGRES_URL") {
        return v;
    }
    let socket = "%2Frun%2Fpostgresql";
    let user = users::get_user_by_uid(users::get_current_uid()).unwrap();
    format!("postgres://{}@{}", user.name().to_str().unwrap(), socket)
}

#[test]
fn test_fdw() {
    let fdw_lib = build_fdw().expect("couldn't compile fdw");

    let tmpdir = tempfile::tempdir().expect("failed to make tempdir");
    let tmplib = tmpdir.path().with_file_name(fdw_lib.file_name().unwrap());
    std::fs::copy(fdw_lib, &tmplib).expect("failed to copy file");
    let fdw_lib_path = tmplib.to_str().unwrap();

    let conn = Connection::connect(get_url(), TlsMode::None).expect("failed to connect");
    // Function names don't need to be escaped the way "$1" would escape them.
    conn.execute(
        format!("DROP FUNCTION IF EXISTS {}() CASCADE", SQL_FUNC_NAME).as_str(),
        &[],
    )
    .expect("failed to drop function");
    conn.execute(
        format!(
            "CREATE FUNCTION {}() RETURNS fdw_handler AS '{}', '{}' LANGUAGE C STRICT",
            SQL_FUNC_NAME, fdw_lib_path, C_FUNC_NAME
        )
        .as_str(),
        &[],
    )
    .expect("failed to create FDW function");
    conn.batch_execute(
        format!(
            "
CREATE FOREIGN DATA WRAPPER dfdw handler {} NO VALIDATOR;

CREATE SERVER df FOREIGN DATA WRAPPER dfdw;

DROP SCHEMA IF EXISTS fdw_test_schema CASCADE;

CREATE SCHEMA fdw_test_schema;

IMPORT FOREIGN SCHEMA test
  FROM SERVER df
  INTO fdw_test_schema;
",
            SQL_FUNC_NAME
        )
        .as_str(),
    )
    .expect("Failed to import foreign schema");

    let rows = conn
        .query("SELECT * FROM fdw_test_schema.mytable", &[])
        .expect("Failed to query FDW");
    assert_eq!(rows.len(), 5);
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(row.len(), 1);
        assert_eq!((i + 1) as i32, row.get::<_, i32>(0))
    }
}
