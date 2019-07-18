extern crate pg_extend;

use pg_extend::pg_create_stmt_bin;

pg_create_stmt_bin!(panicking_pg_create_stmt, longjmping_pg_create_stmt);
