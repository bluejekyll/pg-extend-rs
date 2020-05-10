// Copyright 2018-2019 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

fn lifetime_to_anon(lifetime: &mut syn::Lifetime) {
    let anon_lifetime = syn::Ident::new("_", lifetime.ident.span());
    lifetime.ident = anon_lifetime;
}

fn sl_lifetime_def(lifetime_def: &mut syn::LifetimeDef) {
    lifetime_to_anon(&mut lifetime_def.lifetime);

    for lifetime in &mut lifetime_def.bounds {
        lifetime_to_anon(lifetime);
    }
}

fn sl_type_param_bound(bound: &mut syn::TypeParamBound) {
    use syn::TypeParamBound::*;
    match bound {
        Trait(ref mut trait_bound) => {
            if let Some(bound_lifetimes) = trait_bound.lifetimes.as_mut() {
                for lifetime_def in &mut bound_lifetimes.lifetimes {
                    sl_lifetime_def(lifetime_def);
                }
            };

            sl_path(&mut trait_bound.path);
        }
        Lifetime(ref mut lifetime) => lifetime_to_anon(lifetime),
    }
}

fn sl_generic_argument(args: &mut syn::GenericArgument) {
    use syn::GenericArgument::*;
    match args {
        Lifetime(ref mut lifetime) => lifetime_to_anon(lifetime),
        Type(ref mut ty) => strip_type(ty),
        Binding(ref mut binding) => strip_type(&mut binding.ty),
        Constraint(ref mut constraint) => {
            for mut bound in &mut constraint.bounds {
                sl_type_param_bound(&mut bound);
            }
        }
        Const(expr) => unimplemented!("Const not supported by pg-extern: {:?}", expr),
    }
}

fn sl_path(path: &mut syn::Path) {
    for p in &mut path.segments {
        use syn::PathArguments::*;

        let path_arguments = &mut p.arguments;

        match path_arguments {
            None => (),
            AngleBracketed(ref mut angle_bracketed_generic_arguments) => {
                for generic_argument in &mut angle_bracketed_generic_arguments.args {
                    sl_generic_argument(generic_argument);
                }
            }
            Parenthesized(ref mut parenthesizedgeneric_arguments) => {
                for ty in &mut parenthesizedgeneric_arguments.inputs {
                    strip_type(ty);
                }

                strip_return_type(&mut parenthesizedgeneric_arguments.output);
            }
        }
    }
}

fn sl_type_path(type_path: &mut syn::TypePath) {
    if let Some(ref mut qself) = type_path.qself.as_mut() {
        strip_type(&mut qself.ty);
    };

    sl_path(&mut type_path.path);
}

pub(crate) fn strip_return_type(return_type: &mut syn::ReturnType) {
    use syn::ReturnType::*;
    match return_type {
        Default => (),
        Type(_, ref mut ty) => strip_type(ty),
    }
}

pub(crate) fn strip_type(ty: &mut syn::Type) {
    use syn::Type::*;

    match ty {
        Slice(ref mut type_slice) => strip_type(&mut type_slice.elem),
        Array(type_array) => strip_type(&mut type_array.elem),
        Ptr(type_ptr) => strip_type(&mut type_ptr.elem),
        Reference(type_reference) => strip_type(&mut type_reference.elem),
        BareFn(type_bare_fn) => {
            unimplemented!("BareFn not supported by pg-extern: {:?}", type_bare_fn)
        }
        Never(_type_never) => (),
        Tuple(type_tuple) => {
            for mut i in &mut type_tuple.elems {
                strip_type(&mut i);
            }
        }
        Path(ref mut type_path) => sl_type_path(type_path),
        TraitObject(type_trait_object) => unimplemented!(
            "TraitObject not supported by pg-extern: {:?}",
            type_trait_object
        ),
        ImplTrait(type_impl_trait) => unimplemented!(
            "ImplTrait not supported by pg-extern: {:?}",
            type_impl_trait
        ),
        Paren(type_paren) => unimplemented!("Paren not supported by pg-extern: {:?}", type_paren),
        Group(type_group) => unimplemented!("Group not supported by pg-extern: {:?}", type_group),
        Infer(type_infer) => unimplemented!("Infer not supported by pg-extern: {:?}", type_infer),
        Macro(type_macro) => unimplemented!("Macro not supported by pg-extern: {:?}", type_macro),
        Verbatim(type_verbatim) => {
            unimplemented!("Verbatim not supported by pg-extern: {:?}", type_verbatim)
        }
        t => unimplemented!("Unsupported type: {:?}", t),
    }
}
