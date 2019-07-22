// Copyright 2019 Marti Raudsepp <marti@juffo.org>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate pg_extend;
extern crate pg_extern_attr;

use pg_extend::pg_magic;
use pg_extend::{debug, error, info, log, notice, trace, warn};
use pg_extern_attr::pg_extern;

// This tells Postgres this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

/// An error in PostgreSQL aborts the current statement and (sub)transaction.
#[pg_extern]
fn rs_error(msg: String) {
    error!("{}", msg);
}

/// Log messages in all non-error log levels.
#[pg_extern]
fn rs_log_all() {
    warn!("TEST: This is a warning");
    notice!("TEST: Notice this!");
    info!("TEST: This is an info message");
    log!("TEST: This is an LOG-level message");
    debug!("TEST: This is a debug message");
    trace!("TEST: This is a trace-level message")
}

#[cfg(test)]
mod tests {
    /* Cannot test this module wihtout a PostgreSQL runtime. */
}
