extern crate pg_extend;

use pg_extend::pg_create_stmt_bin;

pg_create_stmt_bin!(get_null_pg_create_stmt, rs_nullif_pg_create_stmt);
