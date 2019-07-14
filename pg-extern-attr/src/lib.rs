// Copyright 2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

mod lifetime;

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::Type;

/// A type that represents that PgAllocator is an argument to the Rust function.
type HasPgAllocatorArg = bool;

fn create_function_params(num_args: usize, has_pg_allocator: HasPgAllocatorArg) -> TokenStream {
    let mut tokens = TokenStream::new();

    // if the allocator is the first arg we want to start at 1
    if has_pg_allocator {
        tokens.extend(quote!(&memory_context,));
    };

    for i in 0..num_args {
        let arg_name = Ident::new(&format!("arg_{}", i), Span::call_site());

        tokens.extend(quote!(
            #arg_name,
        ));
    }

    tokens
}

fn get_arg_types(inputs: &Punctuated<syn::FnArg, Comma>) -> Vec<syn::Type> {
    let mut types = Vec::new();

    for arg in inputs.iter() {
        let arg_type: &syn::Type = match *arg {
            syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
                panic!("self functions not supported")
            }
            syn::FnArg::Inferred(_) => panic!("inferred function parameters not supported"),
            syn::FnArg::Captured(ref captured) => &captured.ty,
            syn::FnArg::Ignored(ref ty) => ty,
        };

        // if it's carrying a lifetime, we're going to replace it with the annonymous one.
        let mut arg_type = arg_type.clone();
        lifetime::strip_type(&mut arg_type);

        types.push(arg_type);
    }

    types
}

/// Check if the argument is the PgAllocator (aka MemoryContext)
fn check_for_pg_allocator(ty: &Type) -> bool {
    // we only accept references, i.e. &PgAllocator
    let type_ref = match ty {
        Type::Reference(type_ref) => type_ref,
        _ => return false,
    };

    // find the path and ident
    match *type_ref.elem {
        Type::Path(ref path) => path
            .path
            .segments
            .iter()
            .last()
            .map_or(false, |p| p.ident.to_string() == stringify!(PgAllocator)),
        _ => false,
    }
}

/// Returns a token stream of all the argument data extracted from the SQL function parameters
///   PgDatums, and converts them to the arg list for the Rust function.
///
/// # Return
///
/// The TokenStream of all the args, and a boolean if the first arg is the PgAllocator
fn extract_arg_data(arg_types: &[Type]) -> (TokenStream, HasPgAllocatorArg) {
    let mut get_args_stream = TokenStream::new();

    // 1 to skip first 0, to use first arg.
    let first_param_pg_allocator = arg_types
        .first()
        .map_or(false, |ty| check_for_pg_allocator(ty));
    let skip_first = if first_param_pg_allocator { 1 } else { 0 };

    for (i, arg_type) in arg_types.iter().skip(skip_first).enumerate() {
        let arg_name = Ident::new(&format!("arg_{}", i), i.span());
        let arg_error = format!("unsupported function argument type for {}", arg_name);

        let get_arg = quote_spanned!( arg_type.span()=>
            let #arg_name: #arg_type = unsafe {
                pg_extend::pg_datum::TryFromPgDatum::try_from(
                    &memory_context,
                    pg_extend::pg_datum::PgDatum::from_raw(
                        &memory_context,
                        *args.next().expect("wrong number of args passed into get_args for args?"),
                        args_null.next().expect("wrong number of args passed into get_args for args_null?")
                    ),
                )
                .expect(#arg_error)
            };
        );

        get_args_stream.extend(get_arg);
    }

    (get_args_stream, first_param_pg_allocator)
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

/// Returns a token stream for the function that creates the function
///
/// # Return
///
/// The TokenStream of all the args, and a boolean if the first arg is the PgAllocator
fn sql_param_types(arg_types: &[Type]) -> (TokenStream, bool) {
    let mut tokens = TokenStream::new();

    // 1 to skip first 0, to use first arg.
    let first_param_pg_allocator = arg_types
        .first()
        .map_or(false, |ty| check_for_pg_allocator(ty));

    let arg_types = if first_param_pg_allocator {
        &arg_types[1..]
    } else {
        arg_types
    };

    for (i, arg_type) in arg_types.iter().enumerate() {
        let sql_name = Ident::new(&format!("sql_{}", i), arg_type.span());

        let sql_param = quote!(
                        #sql_name = pg_extend::pg_type::PgType::from_rust::<String>().as_str(),
        );

        tokens.extend(sql_param);
    }

    (tokens, first_param_pg_allocator)
}

fn sql_return_type(outputs: &syn::ReturnType) -> TokenStream {
    let mut outputs = outputs.clone();
    lifetime::strip_return_type(&mut outputs);

    let ty = match outputs {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    };

    quote_spanned!(ty.span() => pg_extend::pg_type::PgType::from_rust::<#ty>().return_stmt())
}

/// Returns Rust code to figure out if the function takes optional arguments. Functions with
/// non-optional arguments will be declared with the STRICT option. PostgreSQL behavior:
///
/// > If this parameter is specified, the function is not executed when there are null arguments;
/// > instead a null result is assumed automatically.
fn sql_function_options(arg_types: &[Type]) -> TokenStream {
    if arg_types.is_empty() {
        return quote!("",);
    }

    let first_param_pg_allocator = arg_types
        .first()
        .map_or(false, |ty| check_for_pg_allocator(ty));

    let arg_types = if first_param_pg_allocator {
        &arg_types[1..]
    } else {
        arg_types
    };

    if arg_types.is_empty() {
        return quote!("",);
    }

    quote!(
        {
            let optional_args = [ #( <#arg_types>::is_option() ),* ];
            if optional_args.iter().all(|&x| x) { "" }
            else if !optional_args.iter().any(|&x| x) { " STRICT" }
            else {
                panic!("Cannot mix Option and non-Option arguments.");
            }
        },
    )
}

fn impl_info_for_fdw(item: &syn::Item) -> TokenStream {
    let typ = if let syn::Item::Struct(typ) = item {
        typ
    } else {
        panic!("Annotation only supported on structs")
    };

    let mut decl = item.clone().into_token_stream();

    let struct_name = &typ.ident;
    let func_name = syn::Ident::new(&format!("fdw_{}", struct_name), Span::call_site());

    let info_fn = get_info_fn(&func_name);

    let fdw_fn = quote!(
        #[no_mangle]
        pub extern "C" fn #func_name (func_call_info: pg_extend::pg_sys::FunctionCallInfo) -> pg_extend::pg_sys::Datum {
            unsafe { pg_extend::pg_fdw::ForeignWrapper::<#struct_name>::into_datum() }
        }
    );

    let create_sql_name = syn::Ident::new(
        &format!("{}_pg_create_stmt", struct_name),
        Span::call_site(),
    );

    let sql_stmt = format!(
        "
CREATE OR REPLACE FUNCTION {0}() RETURNS fdw_handler AS '{{library_path}}', '{1}' LANGUAGE C STRICT;
CREATE FOREIGN DATA WRAPPER {0} handler {0} NO VALIDATOR;
",
        struct_name, func_name,
    );

    // declare a function that can be used to output a create statement for the externed function
    //   all create statements will be put into a common module for access
    let create_sql_def = quote!(
        #[allow(unused)]
        pub fn #create_sql_name(library_path: &str) -> String {
            use pg_extend::pg_type::PgTypeInfo;

            format!(
                #sql_stmt,
                library_path = library_path
            )
        }
    );

    decl.extend(info_fn);
    decl.extend(create_sql_def);
    decl.extend(fdw_fn);

    decl
}

fn get_info_fn(func_name: &syn::Ident) -> TokenStream {
    let func_info_name = syn::Ident::new(&format!("pg_finfo_{}", func_name), Span::call_site());

    // create the Postgres info
    quote!(
        #[no_mangle]
        pub extern "C" fn #func_info_name () -> &'static pg_extend::pg_sys::Pg_finfo_record {
            const my_finfo: pg_extend::pg_sys::Pg_finfo_record = pg_extend::pg_sys::Pg_finfo_record { api_version: 1 };
            &my_finfo
        }
    )
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
    let mut function = TokenStream::default();

    let func_wrapper_name = syn::Ident::new(&format!("pg_{}", func_name), Span::call_site());
    let func_info = get_info_fn(&func_wrapper_name);
    // join the function information in
    function.extend(func_info);

    let arg_types = get_arg_types(inputs);
    let (get_args_from_datums, has_pg_allocator) = extract_arg_data(&arg_types);
    // remove the optional Rust arguments from the sql argument count
    let num_sql_args = if has_pg_allocator {
        arg_types.len() - 1
    } else {
        arg_types.len()
    };

    let func_params = create_function_params(num_sql_args, has_pg_allocator);

    // wrap the original function in a pg_wrapper function
    let func_wrapper = quote_spanned!( func_name.span() =>
        #[no_mangle]
        #[allow(unused_variables, unused_mut)]
        pub extern "C" fn #func_wrapper_name (func_call_info: pg_extend::pg_sys::FunctionCallInfo) -> pg_extend::pg_sys::Datum {
            use std::panic;
            use pg_extend::pg_alloc::PgAllocator;

            // All params will be in the "current" memory context at the call-site
            let memory_context = PgAllocator::current_context();

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
                    unsafe {
                        result.into_datum()
                    }
                }
                Err(err) => {
                    use std::sync::atomic::compiler_fence;
                    use std::sync::atomic::Ordering;
                    use pg_extend::error;

                    // ensure the return value is null
                    func_info.isnull = pg_extend::pg_bool::Bool::from(true).into();

                    // The Rust code paniced, we need to recover to Postgres via a longjump
                    //   A postgres logging error of Error will do this for us.
                    compiler_fence(Ordering::SeqCst);
                    if let Some(msg) = err.downcast_ref::<&'static str>() {
                        error!("panic executing Rust '{}': {}", stringify!(#func_name), msg);
                    }

                    if let Some(msg) = err.downcast_ref::<String>() {
                        error!("panic executing Rust '{}': {}", stringify!(#func_name), msg);
                    }

                    error!("panic executing Rust '{}'", stringify!(#func_name));

                    unreachable!("log should have longjmped above, this is a bug in pg-extend-rs");
                }
            }
        }
    );

    let create_sql_name =
        syn::Ident::new(&format!("{}_pg_create_stmt", func_name), Span::call_site());

    let (sql_param_types, _has_pg_allocator) = sql_param_types(&arg_types);
    let sql_params = sql_param_list(num_sql_args);
    let sql_options = sql_function_options(&arg_types);
    let sql_return = sql_return_type(output);

    // ret and library_path are replacements at runtime
    let sql_stmt = format!(
        "CREATE or REPLACE FUNCTION {}({}) {{ret}} AS '{{library_path}}', '{}' LANGUAGE C{{opts}};",
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
                opts = #sql_options
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

    // output the original function definition.
    let mut expanded: TokenStream = ast.clone().into_token_stream();

    // Build the impl
    expanded.extend(impl_info_for_fn(&ast));

    // Return the generated impl
    proc_macro::TokenStream::from(expanded)
}

/// An attribute macro for wrapping Rust structs with boiler plate for defining and exposing a foreign data wrapper
/// This is mostly a slimmed down version of pg_extern, with none of the data argument handling.
#[proc_macro_attribute]
#[allow(clippy::needless_pass_by_value)]
pub fn pg_foreignwrapper(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // get a usable token stream
    let ast: syn::Item = parse_macro_input!(item as syn::Item);

    // Build the impl
    let expanded: TokenStream = impl_info_for_fdw(&ast);

    // Return the generated impl
    proc_macro::TokenStream::from(expanded)
}
