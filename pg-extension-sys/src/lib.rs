
#[cfg(feature = "pg_v10")]
pub mod pg_bool;

pub mod pg_datum;
pub mod pg_sys;

use std::os::raw::c_uint;

#[macro_export]
macro_rules! pg_magic {
    (version: $vers:expr) => {
        #[allow(non_upper_case_globals)]
        static mut Pg_magic_data: pg_sys::Pg_magic_struct = pg_sys::Pg_magic_struct {
            len: 0,
            version: 0,
            funcmaxargs: 0,
            indexmaxkeys: 0,
            namedatalen: 0,
            float4byval: 0,
            float8byval: 0,
        };

        #[no_mangle]
        #[allow(non_snake_case)]
        #[allow(unused)]
        #[link_name = "Pg_magic_func"]
        pub extern "C" fn Pg_magic_func() -> &'static pg_sys::Pg_magic_struct {
            use crate::pg_sys::*;
            use std::mem::size_of;
            use std::os::raw::c_int;

            unsafe {
                Pg_magic_data = pg_sys::Pg_magic_struct {
                    len: size_of::<pg_sys::Pg_magic_struct>() as c_int,
                    version: $vers as std::os::raw::c_int / 100,
                    funcmaxargs: FUNC_MAX_ARGS as std::os::raw::c_int,
                    indexmaxkeys: INDEX_MAX_KEYS as std::os::raw::c_int,
                    namedatalen: NAMEDATALEN as std::os::raw::c_int,
                    float4byval: USE_FLOAT4_BYVAL as std::os::raw::c_int,
                    float8byval: USE_FLOAT8_BYVAL as std::os::raw::c_int,
                };

                &Pg_magic_data
            }
        }
    };
}

/// Returns the slice of Datums, and a parallel slice which specifies if the Datum passed in is (SQL) NULL
pub unsafe fn get_args(
    func_call_info: &pg_sys::FunctionCallInfoData,
) -> (&[pg_sys::Datum], &[bool]) {
    use crate::pg_datum::TryFromPgDatum;

    let num_args = func_call_info.nargs as usize;

    let args: &[pg_sys::Datum] = &func_call_info.arg[..num_args];
    let args_null: &[bool] = &func_call_info.argnull[..num_args];

    (args, args_null)
}
