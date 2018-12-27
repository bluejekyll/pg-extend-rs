// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::token::Comma;

fn create_function_params(num_args: usize) -> TokenStream {
    let mut tokens = TokenStream::new();

    for i in 0..num_args {
        let arg_name = Ident::new(&format!("arg_{}", i), Span::call_site());

        tokens.extend(quote!(
            #arg_name,
        ));
    }

    tokens
}

fn extract_arg_data(inputs: &Punctuated<syn::FnArg, Comma>) -> TokenStream {
    let mut get_args_stream = TokenStream::new();

    for (i, arg) in inputs.iter().enumerate() {
        let arg_type: &syn::Type = match *arg {
            syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
                panic!("self functions not supported")
            }
            syn::FnArg::Inferred(_) => panic!("inferred function parameters not supported"),
            syn::FnArg::Captured(ref captured) => &captured.ty,
            syn::FnArg::Ignored(ref ty) => ty,
        };

        let arg_name = Ident::new(&format!("arg_{}", i), Span::call_site());
        let arg_error = format!("unsupported function argument type for {}", arg_name);

        let get_arg = quote!(
            let #arg_name: #arg_type = pg_extend::pg_datum::TryFromPgDatum::try_from(
                pg_extend::pg_datum::PgDatum::from_raw(
                    args[#i],
                    args_null[#i]
                ),
            )
            .expect(#arg_error);
        );

        get_args_stream.extend(get_arg);
    }

    get_args_stream
}

fn impl_info_for_fn(item: &syn::Item) -> TokenStream {
    let func = if let syn::Item::Fn(func) = item {
        func
    } else {
        panic!("annotation only supported on functions");
    };

    let func_name = &func.ident;
    let func_decl = &func.decl;

    if func_decl.variadic.is_some() {
        panic!("variadic functions (...) not supported")
    }

    let generics = &func_decl.generics;
    let inputs = &func_decl.inputs;
    let result = &func_decl.output;
    let func_block = &func.block;

    // declare the function
    let mut function = item.clone().into_token_stream();

    let func_wrapper_name = syn::Ident::new(&format!("pg_{}", func_name), Span::call_site());
    let func_info_name = syn::Ident::new(
        &format!("pg_finfo_{}", func_wrapper_name),
        Span::call_site(),
    );

    // create the postgres info
    let func_info = quote!(
        use pg_extend::pg_sys::Pg_finfo_record;

        #[no_mangle]
        pub extern "C" fn #func_info_name () -> &'static Pg_finfo_record {
            const my_finfo: Pg_finfo_record = Pg_finfo_record { api_version: 1 };
            &my_finfo
        }
    );

    // join the function information in
    function.extend(func_info);

    let get_args_from_datums = extract_arg_data(inputs);
    let func_params = create_function_params(inputs.len());

    // wrap the original function in a pg_wrapper function
    let func_wrapper = quote!(
        #[no_mangle]
        pub extern "C" fn #func_wrapper_name (func_call_info: pg_sys::FunctionCallInfo) -> pg_sys::Datum {
            use std::panic;

            let func_info: &mut pg_extend::pg_sys::FunctionCallInfoData = unsafe {
                func_call_info
                    .as_mut()
                    .expect("func_call_info was unexpectedly NULL")
            };

            let (args, args_null) = unsafe { pg_extend::get_args(func_info) };

            // guard the Postgres process against the panic, and give us an oportunity to cleanup
            let panic_result = panic::catch_unwind(|| {
                // arbitrary Datum conversions occur here, and could panic
                //   so this is inside the catch unwind
                #get_args_from_datums

                // this is the meat of the function call into the extension code
                let result = #func_name(#func_params);

                // arbitrary Rust code could panic, so this is guarded
                pg_extend::pg_datum::PgDatum::from(result)
            });

            // see if we caught a panic
            match panic_result {
                Ok(result) => {
                    // in addition to the null case, we should handle result types probably
                    if result.is_null() {
                        func_info.isnull = true;
                    }

                    // return the datum
                    result.into_datum()
                }
                Err(err) => {
                    // ensure the return value is null
                    func_info.isnull = true;

                    // TODO: anything else to cean up before resuming the panic?
                    panic::resume_unwind(err)
                }
            }
        }
    );

    function.extend(func_wrapper);
    function
}

/// An attribute macro for wrapping Rust functions with boiler plate for defining and
///   calling conventions between Postgres and Rust.
///
///  This mimics the C macro for defining functions
///
/// ```c
/// #define PG_FUNCTION_INFO_V1(funcname) \
/// extern Datum funcname(PG_FUNCTION_ARGS); \
/// extern PGDLLEXPORT const Pg_finfo_record * CppConcat(pg_finfo_,funcname)(void); \
/// const Pg_finfo_record * \
/// CppConcat(pg_finfo_,funcname) (void) \
/// { \
///     static const Pg_finfo_record my_finfo = { 1 }; \
///     return &my_finfo; \
/// } \
/// ```
///
/// # Returns
///
/// The result of this macro will be to produce a new function wrapping the one annotated but prepended with
/// `pg_` to distinquish them and also declares a function for Postgres to get the Function information;
///
/// For example: if the signature `fn add_one(value: i32) -> i32` is annotated, two functions will be produced,
///  the wrapper function with a signature of:
///
/// ```rust,no_run
///  #[no_mangle]
///  pub extern "C" fn pg_add_one(func_call_info: pg_sys::FunctionCallInfo) -> pg_sys::Datum
/// # {
/// # unimplemented!()
/// # }
/// ```
///
/// and the info function with a signature of:
///
/// ```rust,no_run
/// #[no_mangle]
/// pub extern "C" fn pg_finfo_pg_add_one() -> &'static Pg_finfo_record
/// # {
/// # unimplemented!()
/// # }
/// ```
///
#[proc_macro_attribute]
pub fn pg_extern(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // get a usable token stream
    let ast: syn::Item = parse_macro_input!(item as syn::Item);

    // Build the impl
    let expanded: TokenStream = impl_info_for_fn(&ast);

    // Return the generated impl
    proc_macro::TokenStream::from(expanded)
}
