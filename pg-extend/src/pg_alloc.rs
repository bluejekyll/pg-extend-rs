// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A Postgres Allocator

use std::ffi::c_void;
use std::marker::{PhantomData, PhantomPinned};
use std::mem::{self, ManuallyDrop};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use crate::pg_sys;
use std::ptr::NonNull;

/// An allocattor which uses the palloc and pfree functions available from Postgres.
///
/// This is managed by Postgres and guarantees that all memory is freed after a transaction completes.
pub struct PgAllocator(ManuallyDrop<Box<pg_sys::MemoryContextData>>);

impl PgAllocator {
    unsafe fn from_raw(context: *mut pg_sys::MemoryContextData) -> Self {
        Self(ManuallyDrop::new(Box::from_raw(context)))
    }

    pub fn current_context() -> Self {
        unsafe { Self::from_raw(pg_sys::CurrentMemoryContext) }
    }

    pub fn alloc<'mc, T>(&'mc self) -> Pin<Box<PgAllocated<'mc, T>>>
    where
        T: 'mc,
    {
        let size = mem::size_of::<T>();
        // TODO: is there anything we need to do in terms of layout, etc?
        //let ptr = pg_sys::palloc(size) as *mut u8;
        unsafe {
            let ptr = pg_sys::MemoryContextAllocZeroAligned(
                self.0.deref().deref() as *const _ as *mut _,
                size,
            );
            PgAllocated::from_raw(mem::transmute(ptr), self)
        }
    }

    unsafe fn dealloc<'mc, T: ?Sized>(&'mc self, pg_data: *mut T) {
        // TODO: see mctx.c in Postgres' source this probably needs more validation
        let ptr = pg_data as *mut c_void;
        //  pg_sys::pfree(pg_data as *mut c_void)
        let methods = *self.0.methods;
        methods.free_p.expect("free_p is none")(self.0.deref().deref() as *const _ as *mut _, ptr);
    }
}

/// Types that were allocated by Postgres
pub struct PgAllocated<'mc, T: 'mc + ?Sized> {
    inner: Option<ManuallyDrop<Box<T>>>,
    allocator: &'mc PgAllocator,
    _disable_send_sync: PhantomData<NonNull<&'mc T>>,
    _not_unpin: PhantomPinned,
}

impl<'mc, T: 'mc + ?Sized> PgAllocated<'mc, T> {
    pub unsafe fn from_raw(this: *mut T, allocator: &'mc PgAllocator) -> Pin<Box<Self>> {
        let this = PgAllocated {
            inner: Some(ManuallyDrop::new(Box::from_raw(this))),
            allocator,
            _disable_send_sync: PhantomData,
            _not_unpin: PhantomPinned,
        };

        Box::pin(this)
    }

    pub unsafe fn take(mut self) -> *mut T {
        Box::into_raw(ManuallyDrop::into_inner(self.inner.take().unwrap()))
    }
}

impl<'mc, T: 'mc + ?Sized> Deref for PgAllocated<'mc, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("invalid None while PgAllocated is live").deref().deref()
    }
}

impl<'mc, T: 'mc + ?Sized> Drop for PgAllocated<'mc, T> {
    fn drop(&mut self) {
        if let Some(mut inner) = self.inner.take() {
            unsafe {
                // let ptr: *mut T = mem::transmute(inner.deref_mut().deref_mut());
                let ptr: *mut T = inner.deref_mut().deref_mut() as *mut T;
                self.allocator.dealloc(ptr);
            }
        }
    }
}