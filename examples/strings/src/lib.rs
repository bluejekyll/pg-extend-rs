// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.


extern crate pg_extend;
extern crate pg_extern_attr;

use pg_extend::pg_magic;
use pg_extern_attr::pg_extern;
// This tells Postges this library is a Postgres extension
pg_magic!(version: pg_sys::PG_VERSION_NUM);

static mut FUNC_ALLOCATOR: Option<pg_extend::pg_alloc::PgAllocator> = None;

/// The pg_extern attribute wraps the function in the proper functions syntax for C extensions
#[pg_extern]
fn concat_rs(mut a: String, b: String) -> String {
    a.push_str(&b);

    a
}

// #[no_mangle]
// pub extern "C" fn pg_concat_rs(
//     func_call_info: pg_extend::pg_sys::FunctionCallInfo,
// ) -> pg_extend::pg_sys::Datum {
//     use pg_extend::pg_alloc::PgAllocator;
//     use std::panic;

//     let memory_context = unsafe {
//         FUNC_ALLOCATOR = Some(PgAllocator::current_context());
//         FUNC_ALLOCATOR
//             .as_ref()
//             .expect("Global Memory Context not set")
//     };

//     let func_info: &mut pg_extend::pg_sys::FunctionCallInfoData = unsafe {
//         func_call_info
//             .as_mut()
//             .expect("func_call_info was unexpectedly NULL")
//     };
//     let panic_result = panic::catch_unwind(|| {
//         let (mut args, mut args_null) = pg_extend::get_args(func_info);
//         let arg_0: String = {
//             unsafe {
//                 pg_extend::pg_datum::TryFromPgDatum::try_from(
//                     &memory_context,
//                     pg_extend::pg_datum::PgDatum::from_raw(
//                         &memory_context,
//                         *args
//                             .next()
//                             .expect("wrong number of args passed into get_args for args?"),
//                         args_null
//                             .next()
//                             .expect("wrong number of args passed into get_args for args_null?"),
//                     ),
//                 )
//                 .expect("unsupported function argument type for arg_0")
//             }
//         };

//         let arg_1: String = {
//             unsafe {
//                 pg_extend::pg_datum::TryFromPgDatum::try_from(
//                     &memory_context,
//                     pg_extend::pg_datum::PgDatum::from_raw(
//                         &memory_context,
//                         *args
//                             .next()
//                             .expect("wrong number of args passed into get_args for args?"),
//                         args_null
//                             .next()
//                             .expect("wrong number of args passed into get_args for args_null?"),
//                     ),
//                 )
//                 .expect("unsupported function argument type for arg_1")
//             }
//         };
//         let result = concat_rs(arg_0, arg_1);
//         unsafe { pg_extend::pg_datum::PgDatum::from_raw(&memory_context, 0, true) }
//     });
//     match panic_result {
//         Ok(result) => {
//             let isnull: pg_extend::pg_bool::Bool = result.is_null().into();
//             func_info.isnull = isnull.into();
//             unsafe { result.into_datum() }
//         }
//         Err(err) => {
//             use pg_extend::error;
//             use std::sync::atomic::compiler_fence;
//             use std::sync::atomic::Ordering;
//             func_info.isnull = pg_extend::pg_bool::Bool::from(true).into();
//             compiler_fence(Ordering::SeqCst);
//             if let Some(msg) = err.downcast_ref::<&'static str>() {
//                 {
//                     error!("TEST");
//                 };
//             }
//             if let Some(msg) = err.downcast_ref::<String>() {
//                 {
//                     error!("TEST")
//                 };
//             }
//             {
//                 error!("TEST")
//             };
//             {
//                 {
//                     {
//                         panic!("BOOM!")
//                     }
//                 }
//             };
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_rs() {
        assert_eq!(&concat_rs("a".to_string(), "b".to_string()), "ab");
    }
}
