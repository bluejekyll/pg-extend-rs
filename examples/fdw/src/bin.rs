#![cfg(fdw)]

extern crate pg_extend;

use pg_extend::pg_create_stmt_bin;

pg_create_stmt_bin!(DefaultFDW_pg_create_stmt);
