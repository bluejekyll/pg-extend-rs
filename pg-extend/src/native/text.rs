// Copyright 2018-2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::ffi::CString;
use std::ops::Deref;
use std::ptr::NonNull;
use std::str;

use crate::native::VarLenA;
use crate::pg_alloc::{PgAllocated, PgAllocator, RawPtr};
use crate::pg_sys;

/// A zero-overhead view of `text` data from Postgres
pub struct Text<'mc>(PgAllocated<'mc, NonNull<pg_sys::text>>);

impl<'mc> Text<'mc> {
    /// Create from the raw pointer to the Postgres data
    pub unsafe fn from_raw(alloc: &'mc PgAllocator, text_ptr: *mut pg_sys::text) -> Self {
        Text(PgAllocated::from_raw(alloc, text_ptr))
    }

    /// Convert into the underlying pointer
    pub unsafe fn into_ptr(mut self) -> *mut pg_sys::text {
        self.0.take_ptr()
    }

    /// Allocate a new Text data from the CString using the PgAllocator for the Postgres MemoryContext
    pub fn from_cstring(alloc: &'mc PgAllocator, s: CString) -> Self {
        unsafe {
            let text_ptr = { alloc.exec_with_guard(|| pg_sys::cstring_to_text(s.as_ptr())) };

            Text::from_raw(alloc, text_ptr)
        }
    }

    /// Return true if this is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the length of the text data
    pub fn len(&self) -> usize {
        let varlena = unsafe { VarLenA::from_varlena(self.0.as_ref()) };
        varlena.len()
    }

    /// Allocate a new CString, using the PgAllocator for the MemoryContext
    pub fn into_cstring(self, alloc: &'mc PgAllocator) -> PgAllocated<'mc, CString> {
        use std::os::raw::c_char;

        unsafe {
            alloc.exec_with_guard(|| {
                let text_ptr = self.0.as_ptr();

                // from varlena.c
                /*
                 * text_to_cstring
                 *
                 * Create a palloc'd, null-terminated C string from a text value.
                 *
                 * We support being passed a compressed or toasted text value.
                 * This is a bit bogus since such values shouldn't really be referred to as
                 * "text *", but it seems useful for robustness.  If we didn't handle that
                 * case here, we'd need another routine that did, anyway.
                 */
                let cstr = pg_sys::text_to_cstring(text_ptr) as *mut c_char;

                // this is dangerous! it's owned by CString, which is why PgAllocated will
                //  block the dealloc
                PgAllocated::from_raw(alloc, cstr)
            })
        }
    }
}

/// *WARNING* This requires the database to be a UTF-8 locale.
impl<'mc> Deref for Text<'mc> {
    type Target = str;

    fn deref(&self) -> &str {
        // FIXME: this should panic if the DB is not UTF-8.
        unsafe {
            let varlena = VarLenA::from_varlena(self.0.as_ref());
            str::from_utf8_unchecked(&*(varlena.as_slice() as *const [i8] as *const [u8]))
        }
    }
}
