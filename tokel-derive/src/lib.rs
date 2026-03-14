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

/// Evaluates and expands Tokel transformations within the provided token stream.
///
/// This is the primary entry point for the Tokel macro engine. It recursively
/// traverses the provided standard Rust code, looking for expansion blocks
/// denoted by `[< ... >]`.
///
/// When an expansion block is encountered, it is evaluated bottom-up (inside-out).
/// Once the inner tokens are resolved, they are passed through any attached
/// transformer pipelines (e.g., `:concatenate`, `:case[[pascal]]`) to generate
/// the final valid Rust code.
///
/// Standard Rust code outside of the `[< ... >]` blocks is completely ignored
/// and passed through unmodified.
///
/// # Example
///
/// ```rust,ignore
/// tokel::stream! {
///     // Concatenates the text and changes it to pascal case
///     pub struct [< my _ struct >]:concatenate:case[[pascal]] {
///         pub id: usize,
///     }
///
///     // Works inside standard Rust blocks too
///     fn print_id() {
///         let [< my _ variable >]:concatenate = 42;
///     }
/// }
/// ```
#[proc_macro]
pub fn stream(input: TokenStream) -> TokenStream {
    TokenStream::from(
        match syn::parse::<TokelStream>(input)
            .and_then(|target_value| target_value.expand(&mut Session::new()))
        {
            Ok(target_value) => target_value,
            Err(target_error) => target_error.into_compile_error(),
        },
    )
}

/// Evaluates Tokel transformations strictly within the arguments of an attribute.
///
/// Because the Rust compiler evaluates attributes strictly and requires the
/// attached item (like a `struct` or `fn`) to be valid standard Rust code *before*
/// expanding macros, Tokel expansion blocks cannot be used directly in struct
/// or function names without wrapping the entire item in [`tokel::stream!`].
///
/// However, if you only need to manipulate tokens *inside* an attribute
/// (such as `#[doc = ...]`), this macro allows you to do so while keeping the
/// attached Rust item perfectly clean for tools like `rustfmt`.
///
/// # Example
///
/// ```rust,ignore
/// // The `[< ... >]` block inside the doc attribute is expanded,
/// // and the resulting attribute is applied to `MyStruct`.
/// #[tokel::attribute(doc = [< "This is a concatenated " "string!" >]:concatenate)]
/// pub struct MyStruct {
///     pub value: i32,
/// }
/// ```
#[proc_macro_attribute]
pub fn attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    TokenStream::from(match syn::parse::<Meta>(args) {
        Ok(target_meta) => match syn::parse2::<TokelStream>(target_meta.into_token_stream())
            .and_then(|target_value| target_value.expand(&mut Session::new()))
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
