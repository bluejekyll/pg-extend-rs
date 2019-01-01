// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Support for Postgres boolean values

const TRUE_U8: u8 = 1;
const FALSE_U8: u8 = 0;

const TRUE_I8: i8 = 1;
const FALSE_I8: i8 = 0;

const TRUE_CH: char = 1 as char;
const FALSE_CH: char = 0 as char;

/// This type provides conversions for all the possible types that Postgres might use internally for
///   boolean values.
#[derive(Clone, Copy)]
pub struct Bool(bool);

impl From<Bool> for bool {
    fn from(b: Bool) -> Self {
        b.0
    }
}

impl From<bool> for Bool {
    fn from(b: bool) -> Self {
        Bool(b)
    }
}

impl From<Bool> for u8 {
    fn from(b: Bool) -> Self {
        if b.0 {
            TRUE_U8
        } else {
            FALSE_U8
        }
    }
}

impl From<u8> for Bool {
    /// Parse a Bool from a integer.
    ///
    /// Required in the case where bindgen turns a C bool into u8 (i.e. linux)
    ///
    /// ```
    /// extern crate pg_extend;
    /// use pg_extend::pg_bool::Bool;
    /// 
    /// assert_eq!(u8::from(Bool::from(1_u8)), u8::from(Bool::from(true)));
    /// assert_eq!(u8::from(Bool::from(0_u8)), u8::from(Bool::from(false)));
    /// ```
    fn from(i: u8) -> Self {
        Bool(i == TRUE_U8)
    }
}

impl From<Bool> for i8 {
    fn from(b: Bool) -> Self {
        if b.0 {
            TRUE_I8
        } else {
            FALSE_I8
        }
    }
}

impl From<i8> for Bool {
    fn from(i: i8) -> Self {
        Bool(i == TRUE_I8)
    }
}

impl From<Bool> for char {
    fn from(b: Bool) -> Self {
        if b.0 {
            TRUE_CH
        } else {
            FALSE_CH
        }
    }
}

impl From<char> for Bool {
    fn from(i: char) -> Self {
        Bool(i == TRUE_CH)
    }
}