// Copyright 2018-2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::mem;

use crate::pg_sys;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub(crate) enum VarLenA<'a> {
    VarAtt4b(&'a pg_sys::varattrib_4b__bindgen_ty_1),
    VarAtt4bU,
    VarAtt4bC,
    VarAtt1b(&'a pg_sys::varattrib_1b),
    VarAtt1bE,
    VarAttNotPadByte,
}

#[allow(clippy::verbose_bit_mask)]
#[allow(clippy::cast_ptr_alignment)]
impl<'a> VarLenA<'a> {
    /// See postgres.h
    pub(crate) unsafe fn from_varlena(varlena: &'a pg_sys::varlena) -> VarLenA<'a> {
        let varattrib_1b = &*(varlena as *const pg_sys::varlena as *const pg_sys::varattrib_1b);

        if (varattrib_1b.va_header & 0x01) == 0x00 {
            // #define VARATT_IS_4B(PTR) \
            // ((((varattrib_1b *) (PTR))->va_header & 0x01) == 0x00)
            VarLenA::VarAtt4b(
                &*(varlena as *const pg_sys::varlena as *const pg_sys::varattrib_4b__bindgen_ty_1),
            )
        } else if (varattrib_1b.va_header & 0x03) == 0x00 {
            // #define VARATT_IS_4B_U(PTR) \
            // ((((varattrib_1b *) (PTR))->va_header & 0x03) == 0x00)

            VarLenA::VarAtt4bU
        } else if (varattrib_1b.va_header & 0x03) == 0x02 {
            // #define VARATT_IS_4B_C(PTR) \
            // ((((varattrib_1b *) (PTR))->va_header & 0x03) == 0x02)
            VarLenA::VarAtt4bC
        } else if (varattrib_1b.va_header & 0x01) == 0x01 {
            // #define VARATT_IS_1B(PTR) \
            // ((((varattrib_1b *) (PTR))->va_header & 0x01) == 0x01)
            VarLenA::VarAtt1b(&*(varlena as *const pg_sys::varlena as *const pg_sys::varattrib_1b))
        } else if varattrib_1b.va_header == 0x01 {
            // #define VARATT_IS_1B_E(PTR) \
            // ((((varattrib_1b *) (PTR))->va_header) == 0x01)
            VarLenA::VarAtt1bE
        } else {
            /*if *mem::transmute::<&pg_sys::varlena, &u8>(self.as_text()) != 0*/
            // #define VarAttNotPadByte(PTR) \

            VarLenA::VarAttNotPadByte
        }
    }

    pub(crate) fn len(&self) -> usize {
        use VarLenA::*;

        match self {
            // define VARSIZE_4B(PTR) \
            // ((((varattrib_4b *) (PTR))->va_4byte.va_header >> 2) & 0x3FFFFFFF)
            VarAtt4b(varlena) => {
                ((varlena.va_header >> 2) & 0x3FFF_FFFF) as usize
                    - Self::size_of(&varlena.va_header)
            }
            // #define VARSIZE_1B(PTR) \
            // ((((varattrib_1b *) (PTR))->va_header >> 1) & 0x7F)
            VarAtt1b(varlena) => {
                ((varlena.va_header >> 1) & 0x7F) as usize - Self::size_of(&varlena.va_header)
            }
            // #define VARTAG_1B_E(PTR) \
            // (((varattrib_1b_e *) (PTR))->va_tag)
            _ => unimplemented!("this VarLenA type not yet supported: {:?}", self),
        }
    }

    pub(crate) fn as_slice(&self) -> &[std::os::raw::c_char] {
        use VarLenA::*;

        let len = self.len();

        unsafe {
            match self {
                VarAtt4b(varlena) => varlena.va_data.as_slice(len),
                VarAtt1b(varlena) => varlena.va_data.as_slice(len),
                _ => unimplemented!("this VarLenA type not yet supported: {:?}", self),
            }
        }
    }

    fn size_of<T>(_ty: &T) -> usize {
        mem::size_of::<T>()
    }
}
