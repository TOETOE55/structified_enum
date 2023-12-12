#![doc = include_str!("../README.md")]

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashMap;

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use syn::{parse_macro_input, parse_quote, Token};

///
/// transforming a unit-like enum to a struct with its discriminant.
///
/// # Example
///
/// ```rust
/// use structified_enum::structify;
///
/// #[structify]
/// #[repr(u8)]
/// #[derive(Copy, Clone)]
/// enum Foo {
///     A = 0,
///     B,
///     C,
/// }
/// ```
///
/// is equivalent to
///
/// ```rust
/// // #[repr(ty)] -> #[repr(transparent)]
/// #[repr(transparent)]
/// #[derive(Copy, Clone)]
/// struct Foo(u8);
///
/// impl Foo {
///     pub const A: Self = Self(0);
///     pub const B: Self = Self(1);
///     pub const C: Self = Self(2);
///
///     pub fn new(value: u8) -> Self {
///         Self(value)
///     }
///
///     // like `Foo::A as u8`
///     pub fn value(self) -> u8 {
///         self.0
///     }
/// }
/// ```
///
/// # Rules
///
/// 1. `#[structify]` can only be followed by `#[repr]` or `#[derive]`; other attributes are not supported on variants except `#[cfg]`.
/// 2. `#[repr(ty)]` will be converted to `#[repr(transparent)]`.
/// 3. If `#[repr(ty)]` is not applied, the default type of the discriminant is `i32`, and there is no `#[repr(transparent)]`.
/// 4. `#[derive]` only supports `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default`, `Debug`.
/// 5. For `#[derive(Debug)]`, unknown values will be displayed as `"EnumName(value)"`.
/// 6. The `From` trait has been implemented for mutual conversions.
/// 7. Other rules basically maintain consistency with `enum` itself.
///
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
    let enum_clone = r#enum.clone();
    let (reprs, derives, derive_debug, derive_default) = repr_derive(r#enum.attrs)?;
    let (repr_ty, reprs) = repr_ty(reprs)?;

    let (variant_values, default_value) = variants(r#enum.variants)?;

    let vis = r#enum.vis;
    let enum_name = r#enum.ident;

    let mut token_stream = TokenStream::new();

    let struct_item = structify_type(&reprs, &derives, &vis, &enum_name, &repr_ty);
    let inherent_impl = inherent_impl(&enum_name, &repr_ty, &variant_values);
    let from_impl = from_impl(&enum_name, &repr_ty, &variant_values);
    let phantom_enum = phantom_enum(enum_clone);

    struct_item.to_tokens(&mut token_stream);
    inherent_impl.to_tokens(&mut token_stream);
    from_impl.to_tokens(&mut token_stream);
    phantom_enum.to_tokens(&mut token_stream);

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

                Err(meta.error("unsupported derive. It only supports `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default`, `Debug`."))
            })?;
            continue;
        }

        return Err(syn::Error::new(
            attr.span(),
            "unsupported attribute. It only supports `#[repr]` and `#[derive]`",
        ));
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
                return Err(syn::Error::new(
                    path.span(),
                    "conflicting representation hints",
                ));
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
            return Err(syn::Error::new(
                v.span(),
                "unsupported variant. It only supports unit variant",
            ));
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
            default_value = Some(parse_quote!(Self:: #v_name.0));
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
        #[derive(PartialEq,Eq)]
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

            pub const fn value(self) -> #repr_ty {
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
                if self.0 == Self:: #v_name.0 {
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

fn from_impl(
    enum_name: &syn::Ident,
    repr_ty: &syn::Path,
    variant_values: &HashMap<syn::Ident, syn::Expr>,
) -> TokenStream {
    let str_to_value_variants = variant_values.iter().map(|(v_name, _value)| {
        quote! {
            stringify!(#v_name) => Ok(#enum_name::#v_name),
        }
    });
    let value_to_str_conversions = variant_values.iter().map(|(v_name, _value)| {
        quote! {
            #enum_name::#v_name => Ok(stringify!(#v_name).to_string()),
        }
    });
    let error_enum_name = syn::Ident::new(&format!("{}ParseError",enum_name),enum_name.span());
    quote! {
        #[derive(Debug)]
        pub enum #error_enum_name{
            UnrecognizedValue(#repr_ty),
            UnrecognizedString(String),
        }

        impl ::std::error::Error for #error_enum_name {}

        impl ::std::fmt::Display for #error_enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #error_enum_name::UnrecognizedValue(v) => write!(f, "{} Parse Unrecognized value: {}", stringify!(#error_enum_name), v),
                    #error_enum_name::UnrecognizedString(s) => write!(f, "{} Parse Unrecognized string: {}", stringify!(#error_enum_name), s),
                }
            }
        }

        impl ::core::convert::From<#repr_ty> for #enum_name {
            fn from(value: #repr_ty) -> Self {
                Self(value)
            }
        }

        impl ::core::convert::From<#enum_name> for #repr_ty {
            fn from(value: #enum_name) -> Self {
                value.0
            }
        }

        impl ::core::convert::TryFrom<#enum_name> for String {
            type Error = #error_enum_name;
            fn try_from(value: #enum_name) -> ::std::result::Result<String, Self::Error> {
                match value {
                    #(#value_to_str_conversions)*
                    _ => Err(#error_enum_name::UnrecognizedValue(value.0))
                }
            }
        }

        impl ::core::convert::TryFrom<String> for #enum_name {
            type Error = #error_enum_name;
            fn try_from(value: String) -> ::std::result::Result<#enum_name, Self::Error> {
                match value.as_str() {
                    #(#str_to_value_variants)*
                    _ => Err(#error_enum_name::UnrecognizedString(value))
                }
            }
        }
    }
}

// to check discriminant is as enum itself.
fn phantom_enum(mut r#enum: syn::ItemEnum) -> TokenStream {
    r#enum.attrs.clear();
    for v in r#enum.variants.iter_mut() {
        v.attrs.clear();
    }

    quote! {
        const _: () = {
            #r#enum
        };
    }
}
