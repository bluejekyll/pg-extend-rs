// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A Postgres Allocator

use std::alloc::{GlobalAlloc, Layout};
use std::ffi::c_void;

use crate::pg_sys;

/// An allocattor which uses the palloc and pfree functions available from Postgres.
/// 
/// This is managed by Postgres and guarantees that all memory is freed after a transaction completes.
pub struct PgAllocator;

unsafe impl GlobalAlloc for PgAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // TODO: is there anything we need ot do in terms of layout, etc?
        pg_sys::palloc(layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        pg_sys::pfree(dbg!(ptr) as *mut c_void)
    }
}