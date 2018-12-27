// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Error reporting support for Postgres.

use std::os::raw::{c_int, c_char};

use crate::pg_sys;

const ERR_DOMAIN: &[u8] = b"RUST\0";

/// Postgres logging Levels
/// 
/// # Note
/// 
/// Some of these levels effect the status of the connection and transaction in Postgres. Specifically, >= Error will cause
///   the connection and transaction to fail and be reset.
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
    LogServerOnly = pg_sys::LOG_SERVER_ONLY as isize,
    /// Messages specifically requested by user (eg VACUUM VERBOSE output); always sent to client regardless of client_min_messages, but by default not sent to server log.
    Info = pg_sys::INFO as isize,
    /// Helpful messages to users about query operation; sent to client and not to server log by default.
    Notice = pg_sys::NOTICE as isize,
    /// Warnings.  NOTICE is for expected messages like implicit sequence creation by SERIAL. WARNING is for unexpected messages.
    Warning = pg_sys::WARNING as isize,
    /// user error - abort transaction; return to known state
    Error = pg_sys::ERROR as isize,
    ///.fatal error - abort process
    Fatal = pg_sys::FATAL as isize,
    /// take down the other backends with me
    Panic = pg_sys::PANIC as isize,
}

impl From<Level> for c_int {
    fn from(level: Level) -> Self {
        level as isize as c_int
    }
}

// TODO: offer a similar interface to that postgres for multi-log lines?
// TODO: is there a better interface for CStr?
/// log an error to Postgres
pub fn log<T1, T2, T3>(level: Level, file: T1, line: u32, func_name: T2, msg: T3) 
where 
    T1: Into<Vec<u8>>,
    T2: Into<Vec<u8>>,
    T3: Into<Vec<u8>>,
{
    use std::ffi::CString;

    // convert to C ffi
    let file = CString::new(file.into()).expect("this should not fail: file");
    let line = line as c_int; // TODO: check bounds?
    let func_name = CString::new(func_name.into()).expect("this should not fail: func_name");
    let msg = CString::new(msg.into()).or_else(|_| CString::new("failed to convert msg to a CString, check extension code for incompatibly `CString` messages")).expect("this should not fail: msg");

    // these are owned by us
    let file: *const c_char = file.as_ptr();
    let func_name: *const c_char = func_name.as_ptr();
    let msg: *const c_char = msg.as_ptr();

    let errlevel: c_int = c_int::from(level);

    // log the data:
    unsafe {
        if pg_sys::errstart(errlevel, file, line, func_name, ERR_DOMAIN.as_ptr() as *const c_char) {
            let msg_result = pg_sys::errmsg(msg);
            pg_sys::errfinish(msg_result);
        }
    }
}