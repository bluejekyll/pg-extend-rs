// Copyright 2018-2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Postgres Datum conversions for Rust types

use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr::NonNull;

use crate::native::Text;
use crate::pg_alloc::{PgAllocated, PgAllocator};
use crate::pg_bool;
use crate::pg_sys::{self, Datum};

/// A wrapper type for Postgres Datum's.
///
/// This simplifies the semantics around Nullability of the Datum value, and provides conversion tools
///   between Datum and Rust types
#[derive(Clone, Debug)]
pub struct PgDatum<'mc>(Option<Datum>, PhantomData<NonNull<&'mc PgAllocator>>);

impl<'mc> PgDatum<'mc> {
    /// Returns a new PgDatum wrapper for Datatypes used by Postgres.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw<B: Into<pg_bool::Bool>>(
        _memory_context: &'mc PgAllocator,
        datum: Datum,
        is_null: B,
    ) -> PgDatum<'mc> {
        let is_null: pg_bool::Bool = is_null.into();
        let datum = if is_null.into() { None } else { Some(datum) };
        PgDatum(datum, PhantomData)
    }

    /// Returns a new PgDatum wrapper if you already have Option<Datum>
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_option(
        _memory_context: &'mc PgAllocator,
        datum: Option<Datum>,
    ) -> PgDatum<'mc> {
        PgDatum(datum, PhantomData)
    }

    /// Return true if this Datum is None
    ///
    /// # Notes
    ///
    /// This must not panic, this is called directly at the FFI boundary with Postgres, if it panics it will cause
    ///    the full Postgres DB to restart and enter recovery mode.
    pub fn is_null(&self) -> bool {
        self.0.is_none()
    }

    /// Do a direct converstion to the Postgres datum type.
    ///
    /// # Notes
    ///
    /// This must not panic, this is called directly at the FFI boundary with Postgres, if it panics it will cause
    ///    the full Postgres DB to restart and enter recovery mode.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn into_datum(self) -> Datum {
        match self.0 {
            Some(datum) => datum,
            None => 0 as Datum,
        }
    }
}

pub trait PgPrimitiveDatum {}

impl PgPrimitiveDatum for i16 {}
impl PgPrimitiveDatum for i32 {}
impl PgPrimitiveDatum for i64 {}
impl PgPrimitiveDatum for f32 {}
impl PgPrimitiveDatum for f64 {}

/// A trait that allows for conversions between Postgres Datum types and Rust types.
///
/// Only Sized types, that fit in a single Datum, bool, u8 - u64 e.g. Nothing else is
///  safe here.
pub trait TryFromPgDatum<'s>: Sized {
    /// Attempt a conversion to from the Postgres data type into the Rust type
    fn try_from<'mc>(
        memory_context: &'mc PgAllocator,
        datum: PgDatum<'mc>,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        if let Some(datum) = datum.0 {
            unsafe {
                Self::try_cast(memory_context, datum)
            }
        } else {
            Err("datum was NULL")
        }
    }

    /// Used to perform the actual casting. You should not use this directly, but
    /// you should implement it. `datum` is expected to be non-null
    #[doc(hidden)]
    unsafe fn try_cast<'mc>(
        memory_context: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's;
}

impl<'s> TryFromPgDatum<'s> for i16 {
    unsafe fn try_cast<'mc>(
        _: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        Ok(datum as i16)
    }
}

impl From<i16> for PgDatum<'_> {
    fn from(value: i16) -> Self {
        PgDatum(Some(value as Datum), PhantomData)
    }
}

impl<'s> TryFromPgDatum<'s> for f32 {
    unsafe fn try_cast<'mc>(
        _: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        Ok(f32::from_bits(datum as u32))
    }
}

impl From<f32> for PgDatum<'_> {
    fn from(value: f32) -> Self {
        PgDatum(Some(f32::to_bits(value) as Datum), PhantomData)
    }
}

impl<'s> TryFromPgDatum<'s> for f64 {
    unsafe fn try_cast<'mc>(
        _: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        Ok(f64::from_bits(datum as u64))
    }
}

impl From<f64> for PgDatum<'_> {
    fn from(value: f64) -> Self {
        PgDatum(Some(f64::to_bits(value) as Datum), PhantomData)
    }
}

impl<'s> TryFromPgDatum<'s> for i32 {
    unsafe fn try_cast<'mc>(
        _: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        Ok(datum as i32)
    }
}

impl From<i32> for PgDatum<'_> {
    fn from(value: i32) -> Self {
        PgDatum(Some(value as Datum), PhantomData)
    }
}

impl<'s> TryFromPgDatum<'s> for i64 {
    unsafe fn try_cast<'mc>(
        _: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        Ok(datum as i64)
    }
}

impl From<i64> for PgDatum<'_> {
    fn from(value: i64) -> Self {
        assert!(
            std::mem::size_of::<Datum>() >= std::mem::size_of::<i64>(),
            "Datum not large enough for i64 values"
        );
        PgDatum(Some(value as Datum), PhantomData)
    }
}

#[deprecated(note = "String is not Zero cost, please use the CString variant")]
impl<'s> TryFromPgDatum<'s> for String {
    fn try_from<'mc>(
        memory_context: &'mc PgAllocator,
        datum: PgDatum<'mc>,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        let cstr = CString::try_from(memory_context, datum)?;

        cstr.into_string()
            .map_err(|_| "String contained non-utf8 data")
    }

    unsafe fn try_cast<'mc>(_: &'mc PgAllocator, _: Datum) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        unimplemented!("Cast is not implemented for this type");
    }
}

// FIXME: this lifetime is wrong
impl From<String> for PgDatum<'_> {
    fn from(value: String) -> Self {
        let cstr = CString::new(value).expect("This shouldn't fail");
        let ptr: *const c_char = cstr.as_ptr();

        let text = unsafe { crate::guard_pg(|| pg_sys::cstring_to_text(ptr)) };

        PgDatum(Some(text as Datum), PhantomData)
    }
}

#[deprecated(note = "String is not Zero cost, please use the CString variant")]
impl<'s> TryFromPgDatum<'s> for CString {
    unsafe fn try_cast<'mc>(
        _: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        let text_val = datum as *const pg_sys::text;

        crate::guard_pg(|| {
            let val: *mut c_char = pg_sys::text_to_cstring(text_val);
            let cstr = CStr::from_ptr(val).to_owned();

            pg_sys::pfree(val as *mut _);

            Ok(cstr)
        })
    }
}

// FIXME: this lifetime is wrong
impl From<CString> for PgDatum<'_> {
    fn from(value: CString) -> Self {
        let ptr: *const c_char = value.as_ptr();
        let text = unsafe { crate::guard_pg(|| pg_sys::cstring_to_text(ptr)) };

        PgDatum(Some(text as Datum), PhantomData)
    }
}

impl<'s> TryFromPgDatum<'s> for PgAllocated<'s, CString> {
    unsafe fn try_cast<'mc>(
        memory_context: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        let text_val = datum as *const pg_sys::text;
        crate::guard_pg(|| {
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
            let cstr = pg_sys::text_to_cstring(text_val) as *mut c_char;

            // this is dangerous! it's owned by CString, which is why PgAllocated will
            //  block the dealloc
            //let cstr = CString::from_raw(val);
            let allocated = PgAllocated::from_raw(memory_context, cstr);

            Ok(allocated)
        })
    }
}

impl<'s> From<Text<'s>> for PgDatum<'s> {
    fn from(value: Text<'s>) -> Self {
        let ptr = unsafe { value.into_ptr() };
        PgDatum(Some(ptr as Datum), PhantomData)
    }
}

impl<'s> TryFromPgDatum<'s> for Text<'s> {
    unsafe fn try_cast<'mc>(
        memory_context: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        let text_ptr = datum as *const pg_sys::text;

        Ok(Text::from_raw(memory_context, text_ptr as *mut _))
    }
}

impl<'s, T> TryFromPgDatum<'s> for Option<T>
where
    T: 's + TryFromPgDatum<'s>,
{
    fn try_from<'mc>(
        memory_context: &'mc PgAllocator,
        datum: PgDatum<'mc>,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        if datum.is_null() {
            return Ok(None);
        }

        // Value is not NULL: Call try_from(_: &PgAllocator, ) of type T without Optional<>
        let result: Result<T, &'static str> = TryFromPgDatum::try_from(memory_context, datum);

        Ok(Some(result?))
    }

    unsafe fn try_cast<'mc>(_: &'mc PgAllocator, _: Datum) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        unimplemented!("Cast is not implemented for this type");
    }
}

impl<'mc, 's, T> From<Option<T>> for PgDatum<'mc>
where
    'mc: 's,
    T: 's,
    PgDatum<'mc>: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => PgDatum::from(value),
            None => PgDatum(None, PhantomData),
        }
    }
}

impl<'s, T> TryFromPgDatum<'s> for &[T]
where
    T: 's + TryFromPgDatum<'s> + PgPrimitiveDatum,
{
    unsafe fn try_cast<'mc>(
        _: &'mc PgAllocator,
        datum: Datum,
    ) -> Result<Self, &'static str>
    where
        Self: 's,
        'mc: 's,
    {
        let datum = datum as *mut pg_sys::varlena;
        if datum.is_null() {
            return Err("datum was NULL");
        }

        let arr_type = pg_sys::pg_detoast_datum(datum) as *mut pg_sys::ArrayType;

        if (*arr_type).ndim > 1 {
            return Err("argument must be empty or one-dimensional array");
        }

        let mut elmlen: pg_sys::int16 = 0;
        let mut elmbyval: pg_sys::bool_ = 0;
        let mut elmalign: ::std::os::raw::c_char = 0;

        pg_sys::get_typlenbyvalalign((*arr_type).elemtype, &mut elmlen, &mut elmbyval, &mut elmalign);

        let mut nulls = 0 as *mut pg_sys::bool_;
        let mut elements = 0 as *mut Datum;
        let mut nelems: i32 = 0;

        pg_sys::deconstruct_array(arr_type, (*arr_type).elemtype,
            elmlen as i32, elmbyval, elmalign,
            &mut elements, &mut nulls, &mut nelems,
        );

        let datums = std::slice::from_raw_parts(elements as *const Datum, nelems as usize);

        // This is where the conversion from `&[Datum]` is done to `&[T]` by a simple type casting,
        // however, we should use `T::try_cast(&'mc PgAllocator, Datum)` to ignore nulls
        let mem_size_datums = std::mem::size_of_val(datums);
        let datums = if mem_size_datums == 0 {
            std::slice::from_raw_parts(datums.as_ptr() as *const T, 0)
        } else {
            let mem_size_type = std::mem::size_of::<T>();
            assert_eq!(mem_size_datums % mem_size_type, 0);
            std::slice::from_raw_parts(datums.as_ptr() as *const T, mem_size_datums / mem_size_type)
        };

        Ok(datums)
    }
}

impl<'mc, 's, T> From<&[T]> for PgDatum<'mc>
where
    'mc: 's,
    T: 's,
    PgDatum<'mc>: From<T>,
{
    fn from(_value: &[T]) -> Self {
        // TODO
        PgDatum(None, PhantomData)
    }
}

impl From<()> for PgDatum<'static> {
    fn from(_value: ()) -> Self {
        PgDatum(None, PhantomData)
    }
}

// FIXME: wrong lifetime
impl From<Datum> for PgDatum<'static> {
    fn from(datum: Datum) -> PgDatum<'static> {
        PgDatum(Some(datum), PhantomData)
    }
}
