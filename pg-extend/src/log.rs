// Copyright 2019 Marti Raudsepp <marti@juffo.org>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Implements macros for the PostgreSQL logging system.
//!
//! For common log levels, convenient macros are implemented: [`trace!`], [`debug!`], [`log!`],
//! [`info!`], [`notice!`], [`warn!`], [`error!`], [`fatal!`].
//!
//! Other log levels are supported with the generic macro [`pg_log!`]. See the [`Level` enum] for
//! all available log levels.
//!
//! # Note
//!
//! Beware, log levels `ERROR` and higher also abort the current transaction. The PostgreSQL
//! implementation uses exception handling with `longjmp`, which currently has unsafe side-effects.
//!
//! # Example
//!
//! ```rust,no_run
//! use pg_extend::{info, pg_log};
//! use pg_extend::log::Level;
//!
//! info!("{} widgets frobnicated", 10);
//! pg_log!(Level::LogServerOnly, "Big brother is watching {}!", "you");
//! ````
//!
//! # Rust `log` crate
//!
//! The macro names make this mostly a drop-in replacement for the Rust `log` crate. However, there
//! are differences:
//! * Due to PostgreSQL behavior, log levels `ERROR` and higher log levels abort the current
//!   statement and transaction.
//! * `pg_extend` macros do not support the optional `target:` argument.
//! * In the `log` crate, the generic logging macro is called `log!`. However, we use that name as a
//!   specialized macro since PostgreSQL has a `LOG` log level.
//! * `Level` enum contains Postgres-specific log levels; there is no `Level::Trace` for instance.
//!
//! [`trace!`]: ../macro.trace.html
//! [`debug!`]: ../macro.debug.html
//! [`log!`]: ../macro.log.html
//! [`info!`]: ../macro.info.html
//! [`notice!`]: ../macro.notice.html
//! [`warn!`]: ../macro.warn.html
//! [`error!`]: ../macro.error.html
//! [`fatal!`]: ../macro.fatal.html
//! [`pg_log!`]: ../macro.pg_log.html
//! [`Level` enum]: enum.Level.html

use std::ffi::CString;
use std::fmt;
use std::os::raw::{c_char, c_int};

use crate::pg_sys;

/// Postgres logging Levels
///
/// # Note
///
/// Some of these levels effect the status of the connection and transaction in Postgres.
/// Specifically, >= `Error` will cause the connection and transaction to fail and be reset.
#[derive(Clone, Copy)]
pub enum Level {
    /// Debugging messages, in categories of 5 decreasing detail.
    Debug5 = pg_sys::DEBUG5 as isize,
    /// Debugging messages, in categories of 4 decreasing detail.
    Debug4 = pg_sys::DEBUG4 as isize,
    /// Debugging messages, in categories of 3 decreasing detail.
    Debug3 = pg_sys::DEBUG3 as isize,
    /// Debugging messages, in categories of 2 decreasing detail.
    Debug2 = pg_sys::DEBUG2 as isize,
    /// Debugging messages, in categories of 1 decreasing detail.
    Debug1 = pg_sys::DEBUG1 as isize,
    /// Server operational messages; sent only to server log by default.
    Log = pg_sys::LOG as isize,
    /// Same as LOG for server reporting, but never sent to client.
    ///   `CommError` is an alias for this
    #[cfg(not(feature = "postgres-9"))]
    LogServerOnly = pg_sys::LOG_SERVER_ONLY as isize,
    /// Messages specifically requested by user (eg VACUUM VERBOSE output); always sent to client
    /// regardless of client_min_messages, but by default not sent to server log.
    Info = pg_sys::INFO as isize,
    /// Helpful messages to users about query operation; sent to client and not to server log by
    /// default.
    Notice = pg_sys::NOTICE as isize,
    /// Warnings.  NOTICE is for expected messages like implicit sequence creation by SERIAL.
    /// WARNING is for unexpected messages.
    Warning = pg_sys::WARNING as isize,
    /// user error - abort transaction; return to known state
    Error = pg_sys::ERROR as isize,
    /// fatal error - abort process
    Fatal = pg_sys::FATAL as isize,
    /// take down the other backends with me
    Panic = pg_sys::PANIC as isize,
}

impl From<Level> for c_int {
    fn from(level: Level) -> Self {
        level as isize as c_int
    }
}

/// Log a `DEBUG5` level message. This macro is included for easy replacement with Rust "log" crate
/// macros.
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => (
        $crate::pg_log!($crate::log::Level::Debug5, $($arg)*);
    )
}


/// Log a `DEBUG1` level message. These are hidden by default
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (
        $crate::pg_log!($crate::log::Level::Debug1, $($arg)*);
    )
}

/// Logs a `LOG` message. These messages have a high precedence for writing to PostgreSQL server
/// logs but low precedence for sending to the client.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => (
        $crate::pg_log!($crate::log::Level::Log, $($arg)*);
    )
}

/// Log an `INFO` message. Used for information specifically requested by user (eg VACUUM VERBOSE
/// output). These messages are always sent to the client regardless of
/// `client_min_messages` setting, and not to server logs by default.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (
        $crate::pg_log!($crate::log::Level::Info, $($arg)*);
    )
}

/// Log at `NOTICE` level. Use for helpful messages to users about query operation; expected
/// messages like implicit sequence creation by SERIAL.
#[macro_export]
macro_rules! notice {
    ($($arg:tt)*) => (
        $crate::pg_log!($crate::log::Level::Notice, $($arg)*);
    )
}

/// Log at `WARNING` level. Use for messages that are unexpected for the user.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => (
        $crate::pg_log!($crate::log::Level::Warning, $($arg)*);
    )
}

/// Log at `ERROR` level and abort the current query and transaction.
/// Beware! The PostgreSQL implementation uses exception handling with `longjmp`, which currently
/// has unsafe side-effects.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (
        $crate::pg_log!($crate::log::Level::Error, $($arg)*);
    )
}

/// Log a `FATAL` error and exit the current backend process, also closing the database connection.
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => (
        $crate::pg_log!($crate::log::Level::Fatal, $($arg)*);
    )
}

/// Generic logging macro. See the [`Level` enum] for all available log levels.
///
/// Usually one wouldn't call this directly but the more convenient specialized macros.
///
/// # Example
///
/// ```rust,no_run
/// use pg_extend::pg_log;
/// use pg_extend::log::Level;
///
/// pg_log!(Level::LogServerOnly, "Big brother is watching {}!", "you");
/// ````
///
/// [`Level` enum]: enum.Level.html
#[macro_export]
macro_rules! pg_log {
    ($lvl:expr, $($arg:tt)+) => ({
        $crate::log::__private_api_log(
            format_args!($($arg)+),
            $lvl,
            // Construct a tuple; the whole tuple is a compile-time constant.
            &(
                // Construct zero-terminated strings at compile time.
                concat!(module_path!(), "\0") as *const str as *const ::std::os::raw::c_char,
                concat!(file!(), "\0") as *const str as *const ::std::os::raw::c_char,
                line!(),
            ),
        );
    });
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
#[doc(hidden)]
pub fn __private_api_log(
    args: fmt::Arguments,
    level: Level,
    &(module_path, file, line): &(*const c_char, *const c_char, u32),
) {
    let errlevel: c_int = c_int::from(level);
    let line = line as c_int;
    const LOG_DOMAIN: *const c_char = "RUST\0" as *const str as *const c_char;

    // Rust has no "function name" macro, for now we use module path instead.
    // See: https://github.com/rust-lang/rfcs/issues/1743
    let do_log = unsafe {
        crate::guard_pg(|| pg_sys::errstart(errlevel, file, line, module_path, LOG_DOMAIN))
    };

    // If errstart returned false, the message won't be seen by anyone; logging will be skipped
    if pgbool!(do_log) {
        // At this point we format the passed format string `args`; if the log level is suppressed,
        // no string processing needs to take place.
        let msg = format!("{}", args);
        let c_msg = CString::new(msg).or_else(
            |_| CString::new("failed to convert msg to a CString, check extension code for incompatible `CString` messages")
        ).expect("this should not fail: msg");

        unsafe {
            crate::guard_pg(|| {
                let msg_result = pg_sys::errmsg(c_msg.as_ptr());
                pg_sys::errfinish(msg_result);
            })
        }
    }
}
