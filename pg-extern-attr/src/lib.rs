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
                    *args.next().expect("wrong number of args passed into get_args for args?"),
                    args_null.next().expect("wrong number of args passed into get_args for args_null?")
                ),
            )
            .expect(#arg_error);
        );

        get_args_stream.extend(get_arg);
    }

    get_args_stream
}

fn sql_param_list(num_args: usize) -> String {
    let mut tokens = String::new();
    if num_args == 0 {
        return tokens;
    }

    let arg_name = |num: usize| format!("{{sql_{}}}", num);

    for i in 0..(num_args - 1) {
        let arg_name = arg_name(i);
        tokens.push_str(&format!("{},", arg_name));
    }

    let arg_name = arg_name(num_args - 1);
    tokens.push_str(&arg_name);

    tokens
}

fn sql_param_types(inputs: &Punctuated<syn::FnArg, Comma>) -> TokenStream {
    let mut tokens = TokenStream::new();

    for (i, arg) in inputs.iter().enumerate() {
        let arg_type: &syn::Type = match *arg {
            syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
                panic!("self functions not supported")
            }
            syn::FnArg::Inferred(_) => panic!("inferred function parameters not supported"),
            syn::FnArg::Captured(ref captured) => &captured.ty,
            syn::FnArg::Ignored(ref ty) => ty,
        };

        let sql_name = Ident::new(&format!("sql_{}", i), Span::call_site());

        let sql_param = quote!(
            #sql_name = pg_extend::pg_type::PgType::from_rust::<#arg_type>().as_str(),
        );

        tokens.extend(sql_param);
    }

    tokens
}

fn sql_return_type(outputs: &syn::ReturnType) -> TokenStream {
    let ty = match outputs {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    };

    quote!(pg_extend::pg_type::PgType::from_rust::<#ty>().return_stmt())
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

    //let generics = &func_decl.generics;
    let inputs = &func_decl.inputs;
    let output = &func_decl.output;
    //let func_block = &func.block;

    // declare the function
    let mut function = item.clone().into_token_stream();

    let func_wrapper_name = syn::Ident::new(&format!("pg_{}", func_name), Span::call_site());
    let func_info_name = syn::Ident::new(
        &format!("pg_finfo_{}", func_wrapper_name),
        Span::call_site(),
    );

    // create the postgres info
    let func_info = quote!(
        #[no_mangle]
        pub extern "C" fn #func_info_name () -> &'static pg_extend::pg_sys::Pg_finfo_record {
            const my_finfo: pg_extend::pg_sys::Pg_finfo_record = pg_extend::pg_sys::Pg_finfo_record { api_version: 1 };
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
        pub extern "C" fn #func_wrapper_name (func_call_info: pg_extend::pg_sys::FunctionCallInfo) -> pg_extend::pg_sys::Datum {
            use std::panic;

            let func_info: &mut pg_extend::pg_sys::FunctionCallInfoData = unsafe {
                func_call_info
                    .as_mut()
                    .expect("func_call_info was unexpectedly NULL")
            };

            // guard the Postgres process against the panic, and give us an oportunity to cleanup
            let panic_result = panic::catch_unwind(|| {
                // extract the argument list
                let (mut args, mut args_null) = pg_extend::get_args(func_info);

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
                    let isnull: pg_extend::pg_bool::Bool = result.is_null().into();
                    func_info.isnull = isnull.into();

                    // return the datum
                    result.into_datum()
                }
                Err(err) => {
                    // ensure the return value is null
                    func_info.isnull = pg_extend::pg_bool::Bool::from(true).into();

                    // TODO: anything else to cean up before resuming the panic?
                    panic::resume_unwind(err)
                }
            }
        }
    );

    let create_sql_name =
        syn::Ident::new(&format!("{}_pg_create_stmt", func_name), Span::call_site());

    let sql_params = sql_param_list(inputs.len());
    let sql_param_types = sql_param_types(inputs);
    let sql_return = sql_return_type(output);

    // ret and library_path are replacements at runtime
    let sql_stmt = format!(
        "CREATE or REPLACE FUNCTION {}({}) {{ret}} AS '{{library_path}}', '{}' LANGUAGE C STRICT;",
        func_name, sql_params, func_wrapper_name,
    );

    // declare a function that can be used to output a create statement for the externed function
    //   all create statements will be put into a common module for access
    let create_sql_def = quote!(
        #[allow(unused)]
        pub fn #create_sql_name(library_path: &str) -> String {
            use pg_extend::pg_type::PgTypeInfo;

            format!(
                #sql_stmt,
                #sql_param_types
                ret = #sql_return,
                library_path = library_path
            )
        }
    );

    function.extend(func_wrapper);
    function.extend(create_sql_def);

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
/// extern crate pg_extend;
/// use pg_extend::pg_sys;
///
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
/// extern crate pg_extend;
/// use pg_extend::pg_sys;
///
/// #[no_mangle]
/// pub extern "C" fn pg_finfo_pg_add_one() -> &'static pg_sys::Pg_finfo_record
/// # {
/// # unimplemented!()
/// # }
/// ```
///
#[proc_macro_attribute]
#[allow(clippy::needless_pass_by_value)]
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
