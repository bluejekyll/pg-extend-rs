// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(safe_packed_borrows)]
#![allow(clippy::approx_constant)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::new_without_default)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::useless_transmute)]
#![allow(improper_ctypes)]

//! All exported C FFI symbols from Postgres
//!
//! For all function calls into Postgres, they should generally be wrapped in `pg_extend::guard_pg`.

include!(concat!(env!("OUT_DIR"), "/postgres.rs"));

#[cfg(target_os = "linux")]
use std::os::raw::c_int;

#[cfg(target_os = "linux")]
extern "C" {
    #[link_name = "__sigsetjmp"]
    pub fn sigsetjmp(env: *mut sigjmp_buf, savemask: c_int) -> c_int;
}
