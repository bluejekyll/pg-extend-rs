// Copyright 2018-2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::mem;
use std::ffi::CString;
use std::ops::Deref;
use std::ptr::NonNull;
use std::str;

use crate::pg_alloc::{PgAllocated, PgAllocator, RawPtr};
use crate::pg_sys;

pub struct Text<'mc>(PgAllocated<'mc, NonNull<pg_sys::text>>);

impl<'mc> Text<'mc> {
    pub unsafe fn from_raw(alloc: &'mc PgAllocator, text_ptr: *mut pg_sys::text) -> Self {
        Text(PgAllocated::from_raw(alloc, text_ptr))
    }

    pub unsafe fn into_ptr(mut self) -> *mut pg_sys::text {
        self.0.into_ptr()
    }

    pub fn from_cstring(alloc: &'mc PgAllocator, s: CString) -> Self {
        unsafe {
            let text_ptr = { alloc.exec_with_guard(|| pg_sys::cstring_to_text(s.as_ptr())) };

            Text::from_raw(alloc, text_ptr)
        }
    }

    fn as_text(&self) -> &pg_sys::text {
        unsafe { self.0.as_ref() }
    }

    pub fn len(&self) -> usize {
        use std::os::raw::c_char;

        let len_bytes: [c_char; 4] = self.as_text().vl_len_;

        // PG uses the low order two bits as length markers, we know this is 4 bytes... see VARSIZE_4B in postgres.h
        // FIXME: what is the correct endianness here? PG is just straight casting, and then shifting, so big?
        let unshifted = u32::from_ne_bytes(unsafe{ mem::transmute(len_bytes) });
        (unshifted /*>> 2*/) as usize
    }

    pub fn to_cstring(self, alloc: &'mc PgAllocator) -> PgAllocated<'mc, CString> {
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

    // TODO: look into low cost String conversion, requires text to be utf-8
}

impl<'mc> Deref for Text<'mc> {
    type Target = str;

    fn deref(&self) -> &str {
        let len = self.len();
        unsafe { str::from_utf8_unchecked(mem::transmute(self.as_text().vl_dat.as_slice(len))) }
    }
}