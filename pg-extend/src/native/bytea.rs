use std::ops::Deref;
use std::ptr::NonNull;

use crate::native::VarLenA;
use crate::pg_alloc::{PgAllocated, PgAllocator};
use crate::pg_sys;

/// A zero-overhead view of `bytea` data from Postgres
pub struct ByteA<'mc>(PgAllocated<'mc, NonNull<pg_sys::bytea>>);

// :consider would be good to make a derive(FromVarLenA) macro.
impl<'mc> ByteA<'mc> {
    /// Create from the raw pointer to the Postgres data
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw(alloc: &'mc PgAllocator, raw_ptr: *mut pg_sys::bytea) -> Self {
        ByteA(PgAllocated::from_raw(alloc, raw_ptr))
    }

    /// Convert into the underlying pointer
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_ptr(mut self) -> *mut pg_sys::bytea {
        self.0.take_ptr()
    }

    /// Return true if this is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the length of the bytea data
    pub fn len(&self) -> usize {
        let varlena = unsafe { VarLenA::from_varlena(self.0.as_ref()) };
        varlena.len()
    }
}

impl<'mc> Deref for ByteA<'mc> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe {
            let varlena = VarLenA::from_varlena(self.0.as_ref());
            &*(varlena.as_slice() as *const [i8] as *const [u8])
        }
    }
}
