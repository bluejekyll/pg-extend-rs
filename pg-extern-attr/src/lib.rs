#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::parse::ParseStream;
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
            let #arg_name: #arg_type = pg_extension_sys::pg_datum::TryFromPgDatum::try_from(
                pg_extension_sys::pg_datum::PgDatum::from_raw(
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
        use pg_extension_sys::pg_sys::Pg_finfo_record;

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
            let func_info: &mut pg_extension_sys::pg_sys::FunctionCallInfoData = unsafe {
                func_call_info
                    .as_mut()
                    // FIXME: convert panics to Errors
                    .expect("func_call_info was NULL, sorry for killing your DB")
            };

            let (args, args_null) = unsafe { pg_extension_sys::get_args(func_info) };

            #get_args_from_datums

            let result = #func_name(#func_params);
            let result = pg_extension_sys::pg_datum::PgDatum::from(result);

            if result.is_null() {
                func_info.isnull = true;
            }

            result.into_datum()
        }
    );

    function.extend(func_wrapper);
    function
}

/// This mimics the C macro for defining functions
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
