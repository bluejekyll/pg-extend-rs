// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A Postgres Allocator

use std::alloc::{self, GlobalAlloc, Layout};
use std::cell::Cell;
use std::ffi::c_void;
use std::sync::{Mutex, Once};

use crate::pg_sys;

// static INIT_LOCK: Once = Once::new();
// static mut LOCK: Option<Mutex<()>> = None;

thread_local! {
    static IN_PGCONTEXT: Cell<bool> = Cell::new(false);
}

/// An allocattor which uses the palloc and pfree functions available from Postgres.
///
/// This is managed by Postgres and guarantees that all memory is freed after a transaction completes.
pub struct PgAllocator;

impl PgAllocator {
    //     /// Initializes the PgAllocator, this must be called at least once before Rust code is executed;
    //     pub fn init() {
    //         INIT_LOCK.call_once(|| unsafe {
    //             LOCK = Some(Mutex::new(()));
    //         })
    //     }
}

/// Initializes the PgAllocator, this must be called at least once before Rust code is executed;
    pub fn set_in_pgcontext(b: bool) {
        // INIT_LOCK.call_once(|| unsafe {
        //     LOCK = Some(Mutex::new(()));
        // })

        IN_PGCONTEXT.with(|in_pgcontext| in_pgcontext.set(b));
    }

    pub fn is_in_pgcontext() -> bool {
        IN_PGCONTEXT.with(Cell::get)
    }

unsafe impl GlobalAlloc for PgAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // PgAllocator::init();

        if IN_PGCONTEXT.with(Cell::get) {
            // Postgres is not threadsafe, this makes sure that any Rust threads will only use
            //  the allocator in a thread safe manner
            // let locked = LOCK
            //     .as_ref()
            //     .expect("PgAllocator was not initialized")
            //     .lock()
            //     .expect("LOCK was poisoned");

            // TODO: is there anything we need ot do in terms of layout, etc?
            let allocation = pg_sys::palloc(layout.size()) as *mut u8;
            // encforce lock until after allocation
            // drop(locked);

            allocation
        } else {
            alloc::System.alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // PgAllocator::init();

        if IN_PGCONTEXT.with(Cell::get) {
            // Postgres is not threadsafe, this makes sure that any Rust threads will only use
            //  the allocator in a thread safe manner
            // let locked = LOCK
            //     .as_ref()
            //     .expect("PgAllocator was not initialized")
            //     .lock()
            //     .expect("LOCK was poisoned");

            pg_sys::pfree(ptr as *mut c_void);

        // encforce lock until after deallocation
        // drop(locked);
        } else {
            alloc::System.dealloc(ptr, layout)
        }
    }
}
