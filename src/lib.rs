//! # axum_thiserror
//! `axum_thiserror` is a library that offers a procedural macro to allow `thiserror` error types to be used as `axum` responses.
//! ## Usage
//! Add the library to your current project using Cargo:
//! ```bash
//! cargo add axum_thiserror
//! ```
//! Then you can create a basic `thiserror` error:
//! ```rust,ignore
//! #[derive(Error, Debug)]
//! pub enum UserCreateError {
//!   #[error("User {0} already exists")]
//!   UserAlreadyExists(String),
//! }
//! ```
//! Now you can use `axum_thiserror` to implement `IntoResponse` on your error. By
//! default (the `schemars` feature), the response is the variant's status code with
//! `axum::Json(self)` as the body, so the enum must also derive `serde::Serialize`
//! and `schemars::JsonSchema` (add `serde` and `schemars` to your dependencies) —
//! a missing implementation is a compile error:
//! ```rust,ignore
//! #[derive(Error, Debug, Serialize, JsonSchema, ErrorStatus)]
//! pub enum UserCreateError {
//!   #[error("User {0} already exists")]
//!   #[status(StatusCode::CONFLICT)]
//!   UserAlreadyExists(String),
//! }
//! ```
//! Deriving `JsonSchema` gives a schema matching the response body exactly, for use
//! with OpenAPI tooling such as `aide` or `utoipa`.
//! ## Plain-text responses
//! Disable default features (`default-features = false`) to simply implement
//! `IntoResponse` with the variant's status code and the plain-text `#[error(...)]`
//! message as the body — no extra derives needed:
//! ```rust,ignore
//! #[derive(Error, Debug, ErrorStatus)]
//! pub enum UserCreateError {
//!   #[error("User {0} already exists")]
//!   #[status(StatusCode::CONFLICT)]
//!   UserAlreadyExists(String),
//! }
//! ```
//! ## License
//! This project is licensed under the [MIT License](LICENSE).

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Data, DeriveInput, LitInt, Meta, Path, Variant, parse_macro_input, punctuated::Punctuated,
    spanned::Spanned, token::Comma,
};

/// A derivation that implements the `IntoResponse` trait for a specific attribute.
#[proc_macro_derive(ErrorStatus, attributes(status))]
pub fn derive_error_status(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = parse_macro_input!(input);
    let enum_ident = ast.ident;
    let status_arms: Punctuated<TokenStream, Comma> = match ast.data {
        Data::Enum(data) => data.variants,
        _ => panic!(
            "#[derive(ErrorStatus)] is only available for enums, other types are not supported."
        ),
    }
    .iter()
    .map(impl_enum_variant)
    .collect();

    #[cfg(feature = "schemars")]
    let body = quote!(axum::Json(self));
    #[cfg(not(feature = "schemars"))]
    let body = quote!(format!("{}", self));

    #[cfg(feature = "schemars")]
    let bounds_assertion = quote_spanned! { enum_ident.span() =>
        const _: fn() = || {
            fn assert_json_error_response<T: ::serde::Serialize + ::schemars::JsonSchema>() {}
            assert_json_error_response::<#enum_ident>();
        };
    };
    #[cfg(not(feature = "schemars"))]
    let bounds_assertion = quote!();

    quote! {
        #bounds_assertion

        impl axum::response::IntoResponse for #enum_ident {
            fn into_response(self) -> axum::response::Response {
                let status: axum::http::StatusCode = match &self {
                    #status_arms
                };
                axum::response::IntoResponse::into_response((status, #body))
            }
        }
    }
    .into()
}

fn impl_enum_variant(input: &Variant) -> TokenStream {
    let status_code = find_status_code(input);
    let case = if input.fields.is_empty() {
        case_empty_fields(input)
    } else if input.fields.iter().filter(|f| f.ident.is_none()).count() > 0 {
        case_unnamed_fields(input)
    } else {
        case_named_fields(input)
    };
    quote! {
        Self::#case => #status_code
    }
}

fn case_empty_fields(input: &Variant) -> TokenStream {
    let ident = &input.ident;
    quote!(#ident)
}

fn case_unnamed_fields(input: &Variant) -> TokenStream {
    let ident = &input.ident;
    quote!(#ident( .. ))
}

fn case_named_fields(input: &Variant) -> TokenStream {
    let ident = &input.ident;
    quote!(#ident { .. })
}

fn find_status_code(input: &Variant) -> TokenStream {
    match input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("status"))
    {
        Some(attr) => match &attr.meta {
            Meta::List(l) => {
                if let Ok(number) = l.parse_args::<LitInt>() {
                    quote! {
                        axum::http::StatusCode::from_u16(#number as u16).unwrap()
                    }
                } else if let Ok(expr) = l.parse_args::<Path>() {
                    quote! {
                        #expr
                    }
                } else {
                    quote_spanned!(l.span() => compile_error!("Only #[status(StatusCode)] or #[status(u16)] syntaxe is supported"))
                }
            }
            _ => {
                quote_spanned! { attr.span() => compile_error!("Only #[status(StatusCode)] or #[status(u16)] syntaxe is supported") }
            }
        },
        None => {
            quote_spanned! { input.span() => compile_error!("Each enum variant should have a status code provided using the #[status()] attribute") }
        }
    }
}
