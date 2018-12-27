// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! only needed for pg v. 10?

use crate::pg_sys::bool_;

const TRUE_: bool_ = 1;
const FALSE_: bool_ = 0;

pub struct Bool(bool_);

impl Bool {
    pub fn from_raw(b: bool_) -> Self {
        Bool(b)
    }

    pub fn into_bool(self) -> bool_ {
        self.0
    }
}

impl From<Bool> for bool {
    fn from(b: Bool) -> Self {
        b.0 == TRUE_
    }
}

impl From<bool> for Bool {
    fn from(b: bool) -> Self {
        if b {
            Bool(TRUE_)
        } else {
            Bool(FALSE_)
        }
    }
}
