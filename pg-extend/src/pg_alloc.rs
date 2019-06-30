// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A Postgres Allocator

use std::alloc::{GlobalAlloc, Layout};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop};
use std::ops::DerefMut;

use crate::pg_sys;
use std::ptr::NonNull;


/// An allocattor which uses the palloc and pfree functions available from Postgres.
///
/// This is managed by Postgres and guarantees that all memory is freed after a transaction completes.
pub struct PgAllocator;

impl PgAllocator {
    unsafe fn alloc<'mc, T>(&'mc self) -> PgAllocated<'mc, T>
    where
        T: 'mc,
    {
        let size = mem::size_of::<T>();
        // TODO: is there anything we need to do in terms of layout, etc?
        let ptr = pg_sys::palloc(size) as *mut u8;
        PgAllocated::from_raw(mem::transmute(ptr), self)
    }

    unsafe fn dealloc<'mc, T: ?Sized>(&'mc self, pg_data: *mut T) {
        pg_sys::pfree(pg_data as *mut c_void)
    }
}

/// Types that were allocated by Postgres
pub struct PgAllocated<'mc, T: 'mc + ?Sized> {
    inner: ManuallyDrop<Box<T>>,
    allocator: &'mc PgAllocator,
    _disable_send_sync: PhantomData<NonNull<&'mc T>>,
}

impl<'mc, T: 'mc + ?Sized> PgAllocated<'mc, T> {
    pub unsafe fn from_raw(this: *mut T, allocator: &'mc PgAllocator) -> Self {
        PgAllocated {
            inner: ManuallyDrop::new(Box::from_raw(this)),
            allocator,
            _disable_send_sync: PhantomData,
        }
    }

    pub unsafe fn take(self) -> *mut T {

        Box::into_raw(ManuallyDrop::into_inner(self.inner))
    }
}

impl<'mc, T: 'mc + ?Sized> Drop for PgAllocated<'mc, T> {
    fn drop(&mut self) {
        unsafe {
            let ptr: *mut T = mem::transmute(self.inner.deref_mut().deref_mut());
            self.allocator.dealloc(ptr);
        }
    }
}