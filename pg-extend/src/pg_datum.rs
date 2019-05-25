// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Postgres Datum conversions for Rust types

use crate::pg_sys::{self, Datum};
use crate::pg_bool;

use std::ffi::{CStr, CString};

/// A wrapper type for Postgres Datum's.
///
/// This simplifies the semantics around Nullability of the Datum value, and provides conversion tools
///   between Datum and Rust types
#[derive(Clone, Debug)]
pub struct PgDatum(Option<Datum>);

impl PgDatum {
    /// Returns a new PgDatum wrapper for Datatypes used by Postgres.
    pub fn from_raw<B: Into<pg_bool::Bool>>(datum: Datum, is_null: B) -> Self {
        let is_null: pg_bool::Bool = is_null.into();
        let datum = if is_null.into() { None } else { Some(datum) };
        PgDatum(datum)
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
    pub fn into_datum(self) -> Datum {
        match self.0 {
            Some(datum) => datum,
            None => 0 as Datum,
        }
    }
}

/// A trait that allows for conversions between Postgres Datum types and Rust types.
pub trait TryFromPgDatum: Sized {
    /// Attempt a conversion to from the Postgres data type into the Rust type
    fn try_from(datum: PgDatum) -> Result<Self, &'static str>;
}

impl TryFromPgDatum for i16 {
    fn try_from(datum: PgDatum) -> Result<Self, &'static str> {
        if let Some(datum) = datum.0 {
            Ok(datum as i16)
        } else {
            Err("datum was NULL")
        }
    }
}

impl From<i16> for PgDatum {
    fn from(value: i16) -> Self {
        PgDatum(Some(value as Datum))
    }
}

impl TryFromPgDatum for i32 {
    fn try_from(datum: PgDatum) -> Result<Self, &'static str> {
        if let Some(datum) = datum.0 {
            Ok(datum as i32)
        } else {
            Err("datum was NULL")
        }
    }
}

impl From<i32> for PgDatum {
    fn from(value: i32) -> Self {
        PgDatum(Some(value as Datum))
    }
}

impl TryFromPgDatum for i64 {
    fn try_from(datum: PgDatum) -> Result<Self, &'static str> {
        assert!(
            std::mem::size_of::<Datum>() >= std::mem::size_of::<i64>(),
            "Datum not large enough for i64 values"
        );
        if let Some(datum) = datum.0 {
            Ok(datum as i64)
        } else {
            Err("datum was NULL")
        }
    }
}

impl From<i64> for PgDatum {
    fn from(value: i64) -> Self {
        assert!(
            std::mem::size_of::<Datum>() >= std::mem::size_of::<i64>(),
            "Datum not large enough for i64 values"
        );
        PgDatum(Some(value as Datum))
    }
}

impl TryFromPgDatum for String {
    fn try_from(datum: PgDatum) -> Result<Self, &'static str> {
        let cstr = CString::try_from(datum)?;

        cstr.into_string().map_err(|_| "String contained non-utf8 data")
    }
}

impl From<String> for PgDatum {
    fn from(value: String) -> Self {
        use std::os::raw::c_char;

        let cstr = CString::new(value).expect("This shouldn't fail");
        let ptr: *const c_char = cstr.as_ptr();

        let text = unsafe { pg_sys::cstring_to_text(ptr) };

        PgDatum(Some(text as Datum))
    }
}

impl TryFromPgDatum for CString {
    fn try_from(datum: PgDatum) -> Result<Self, &'static str> {
        use std::os::raw::c_char;

        if let Some(datum) = datum.0 {
            let text_val = datum as *const pg_sys::text;

            unsafe {
                let val: *mut c_char = pg_sys::text_to_cstring(text_val);
                let cstr = CStr::from_ptr(val).to_owned();

                pg_sys::pfree(val as *mut _);

                Ok(cstr)
            }
        } else {
            Err("datum was NULL")
        }
    }
}

impl From<CString> for PgDatum {
    fn from(value: CString) -> Self {
        use std::os::raw::c_char;

        let ptr: *const c_char = value.as_ptr();
        let text = unsafe { pg_sys::cstring_to_text(ptr) };

        PgDatum(Some(text as Datum))
    }
}

impl<T> TryFromPgDatum for Option<T> where T: TryFromPgDatum {
    fn try_from(datum: PgDatum) -> Result<Self, &'static str> {
        if datum.is_null() {
            return Ok(None);
        }

        // Value is not NULL: Call try_from() of type T without Optional<>
        let result: Result<T, &'static str> = TryFromPgDatum::try_from(datum);

        Ok(Some(result?))
    }
}

impl<T> From<Option<T>> for PgDatum where PgDatum: From<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => PgDatum::from(value),
            None => PgDatum(None)
        }
    }
}

impl From<()> for PgDatum {
    fn from(_value: ()) -> Self {
        PgDatum(None)
    }
}

impl From<Datum> for PgDatum {
    fn from(datum: Datum) -> PgDatum {
        PgDatum(Some(datum))
    }
}
