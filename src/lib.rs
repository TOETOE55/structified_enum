#![doc = include_str!("../README.md")]

use proc_macro2::TokenStream;
use quote::ToTokens;
use std::collections::HashMap;

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use syn::{parse_macro_input, parse_quote, Token};

/// # TODO
// `#[structify] #[repr()] #[derive()] enum Foo {..}`
#[proc_macro_attribute]
pub fn structify(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let enum_input = parse_macro_input!(item as syn::ItemEnum);
    structify_impl(enum_input)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}

fn structify_impl(r#enum: syn::ItemEnum) -> syn::Result<TokenStream> {
    let (reprs, derives, derive_debug, derive_default) = repr_derive(r#enum.attrs)?;
    let (repr_ty, reprs) = repr_ty(reprs)?;

    let (variant_values, default_value) = variants(r#enum.variants)?;

    let vis = r#enum.vis;
    let enum_name = r#enum.ident;

    let mut token_stream = TokenStream::new();

    let struct_item = structify_type(&reprs, &derives, &vis, &enum_name, &repr_ty);
    let inherent_impl = inherent_impl(&enum_name, &repr_ty, &variant_values);

    struct_item.to_tokens(&mut token_stream);
    inherent_impl.to_tokens(&mut token_stream);

    if derive_debug {
        let debug_impl = debug_impl(&enum_name, &variant_values);
        debug_impl.to_tokens(&mut token_stream);
    }

    if derive_default {
        let default_impl = default_impl(&enum_name, &default_value);
        default_impl.to_tokens(&mut token_stream);
    }

    Ok(token_stream)
}

fn repr_derive(
    enum_attrs: Vec<syn::Attribute>,
) -> syn::Result<(
    // repr(...)
    Vec<syn::Meta>,
    // derive(...)
    Vec<syn::Path>,
    // derive Debug
    bool,
    // derive Default
    bool,
)> {
    let mut reprs = vec![];
    let mut derives = vec![];
    let mut derive_debug = false;
    let mut derive_default = false;
    for attr in enum_attrs {
        // cfg会最先展开，但会把attribute留在上面
        if attr.path().is_ident("cfg") {
            continue;
        }

        if attr.path().is_ident("repr") {
            reprs.extend(
                attr.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)?,
            );
            continue;
        }

        if attr.path().is_ident("derive") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("Clone")
                    || meta.path.is_ident("Copy")
                    || meta.path.is_ident("Eq")
                    || meta.path.is_ident("PartialEq")
                    || meta.path.is_ident("Ord")
                    || meta.path.is_ident("PartialOrd")
                    || meta.path.is_ident("Hash")
                {
                    derives.push(meta.path);
                    return Ok(());
                }

                // Debug trait要重新实现
                if meta.path.is_ident("Debug") {
                    derive_debug = true;
                    return Ok(());
                }

                // Default trait要重新实现
                if meta.path.is_ident("Default") {
                    derive_default = true;
                    return Ok(());
                }

                Err(meta.error("unsupported derive"))
            })?;
            continue;
        }

        return Err(syn::Error::new(attr.span(), "unsupported attribute"));
    }

    Ok((reprs, derives, derive_debug, derive_default))
}

fn repr_ty(reprs: Vec<syn::Meta>) -> syn::Result<(syn::Path, Vec<syn::Meta>)> {
    let mut repr_ty = None;
    let has_transparent = reprs.contains(&parse_quote!(transparent));
    let mut new_reprs = vec![];
    for repr in reprs {
        let syn::Meta::Path(path) = &repr else {
            new_reprs.push(repr);
            continue;
        };

        if path.is_ident("i8")
            || path.is_ident("u8")
            || path.is_ident("i16")
            || path.is_ident("u16")
            || path.is_ident("i32")
            || path.is_ident("u32")
            || path.is_ident("i64")
            || path.is_ident("u64")
            || path.is_ident("i128")
            || path.is_ident("u128")
            || path.is_ident("isize")
            || path.is_ident("usize")
        {
            if repr_ty.is_none() {
                if !has_transparent {
                    repr_ty = Some(path.clone());
                }
                continue;
            } else {
                return Err(syn::Error::new(path.span(), "conflicting representation hints"));
            }
        }

        new_reprs.push(repr);
    }

    Ok((repr_ty.unwrap_or_else(|| parse_quote!(i32)), new_reprs))
}

fn variants(
    enum_variants: impl IntoIterator<Item = syn::Variant>,
) -> syn::Result<(
    // variant name -> value
    HashMap<syn::Ident, syn::Expr>,
    // default value
    syn::Expr,
)> {
    let mut variant_values = HashMap::new();
    let mut value: syn::Expr = parse_quote!(0);
    let mut default_value = None;
    for v in enum_variants {
        // 仅支持unit-like enum
        if !matches!(v.fields, syn::Fields::Unit) {
            return Err(syn::Error::new(v.span(), "unsupported variant"));
        }

        // cfg会最先展开，但会把attribute留在上面
        for v_attr in v.attrs {
            if v_attr.path().is_ident("cfg") {
                continue;
            }

            return Err(syn::Error::new(v_attr.span(), "unsupported attribute"));
        }

        if let Some((_, expr)) = v.discriminant {
            value = expr
        }

        if default_value.is_none() {
            let v_name = v.ident.clone();
            default_value = Some(parse_quote!(Self:: #v_name.value()));
        }
        variant_values.insert(v.ident, value.clone());
        value = parse_quote! { #value + 1 };
    }

    Ok((
        variant_values,
        default_value.unwrap_or_else(|| parse_quote!(0)),
    ))
}

fn structify_type(
    reprs: &[syn::Meta],
    derives: &[syn::Path],
    vis: &syn::Visibility,
    enum_name: &syn::Ident,
    repr_ty: &syn::Path,
) -> syn::ItemStruct {
    parse_quote! {
        #(#[repr(#reprs)])*
        #(#[derive(#derives)])*
        #vis struct #enum_name(#repr_ty);
    }
}

fn inherent_impl(
    enum_name: &syn::Ident,
    repr_ty: &syn::Path,
    variant_values: &HashMap<syn::Ident, syn::Expr>,
) -> syn::ItemImpl {
    let const_variants: Vec<syn::ItemConst> = variant_values
        .iter()
        .map(|(v_name, value)| {
            parse_quote! {
                pub const #v_name: Self = Self(#value);
            }
        })
        .collect();

    parse_quote! {
        impl #enum_name {
            #(#const_variants)*

            pub const fn new(value: #repr_ty) -> Self {
                Self(value)
            }

            pub const fn value(&self) -> #repr_ty {
                self.0
            }
        }
    }
}

fn debug_impl(
    enum_name: &syn::Ident,
    variant_values: &HashMap<syn::Ident, syn::Expr>,
) -> syn::ItemImpl {
    let stmts: Vec<syn::ExprIf> = variant_values
        .keys()
        .map(|v_name| {
            parse_quote! {
                if self.value() == Self:: #v_name.value() {
                    return f.debug_struct(stringify!(#v_name)).finish();
                }
            }
        })
        .collect();

    parse_quote! {
        impl ::core::fmt::Debug for #enum_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #(#stmts)*

                f.debug_tuple(stringify!(#enum_name))
                    .field(&self.0)
                    .finish()
            }
        }
    }
}

fn default_impl(enum_name: &syn::Ident, default_value: &syn::Expr) -> syn::ItemImpl {
    parse_quote! {
        impl ::core::default::Default for #enum_name {
            fn default() -> Self {
                Self(#default_value)
            }
        }
    }
}
