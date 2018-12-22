#[macro_use]
extern crate pg_extern_macro;

pub mod pg_bool;
pub mod pg_datum;
pub mod pg_sys;

use std::os::raw::c_uint;

use crate::pg_datum::TryFromPgDatum;

#[macro_export]
macro_rules! pg_module {
    (version: $vers:expr) => {
        #[allow(non_upper_case_globals)]
        static mut Pg_magic_data: pg_sys::Pg_magic_struct = pg_sys::Pg_magic_struct {
            len: 0 as std::os::raw::c_int,
            version: $vers as std::os::raw::c_int,
            funcmaxargs: 100,
            indexmaxkeys: 32,
            namedatalen: 64,
            float4byval: 1,
            float8byval: 1,
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

pg_module!(version: pg_sys::PG_VERSION_NUM);

pub unsafe fn get_args(
    func_call_info: &pg_sys::FunctionCallInfoData,
) -> (&[pg_sys::Datum], &[pg_sys::bool_]) {
    let num_args = func_call_info.nargs as usize;
    let args: &[pg_sys::Datum] = std::slice::from_raw_parts(func_call_info.arg, num_args);
    let args_null: &[pg_sys::bool_] = std::slice::from_raw_parts(func_call_info.argnull, num_args);

    (args, args_null)
}

#[pg_extern]
fn add_one(value: i32) -> i32 {
    (value + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_one() {
        assert_eq!(add_one(1), 2);
    }
}
