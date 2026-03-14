#![forbid(
    clippy::all,
    clippy::perf,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::panic,
    clippy::pedantic,
    rustdoc::all,
    unsafe_code
)]
//! # `tokel-derive`
//!
//! This is an internal crate used to expose the standard procedural macros.
//!
//! Tokel exposes a serie of procedural macros that can be useful in distinct circumstances:
//!
//! * [`stream`]: to act upon an arbitrary token-stream
//! * [`attribute`]: to act upon an attribute attached to a Rust item.
//!
//! ---
//!
//! *The following is the main `tokel` workspace documentation:*
//!
#![doc = include_str!("../../README.md")]

use proc_macro::TokenStream;

use quote::{ToTokens, quote};

use syn::Meta;

use tokel_engine::{expand::Expand, session::Session, syntax::TokelStream};

#[proc_macro]
pub fn stream(input: TokenStream) -> TokenStream {
    TokenStream::from(
        match syn::parse::<TokelStream>(input)
            .map(|target_value| target_value.expand(&mut Session::new()))
            .flatten()
        {
            Ok(target_value) => target_value,
            Err(target_error) => target_error.into_compile_error(),
        },
    )
}

#[proc_macro_attribute]
pub fn attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    TokenStream::from(match syn::parse::<Meta>(args) {
        Ok(target_meta) => match syn::parse2::<TokelStream>(target_meta.into_token_stream())
            .map(|target_value| target_value.expand(&mut Session::new()))
            .flatten()
        {
            Ok(target_value) => quote! {
                #[#target_value]
                #input
            },
            Err(target_error) => target_error.into_compile_error(),
        },
        Err(target_error) => target_error.into_compile_error(),
    })
}
