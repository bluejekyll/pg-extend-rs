// Copyright 2018-2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Module for native Postgres types.
//! 
//! These shoudl be near zero overhead types, exposed from Postgres and able to be directly used.

mod text;
mod varlena;

pub use text::Text;
pub(crate) use varlena::VarLenA;