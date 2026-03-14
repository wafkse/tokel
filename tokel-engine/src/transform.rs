//! The transformer pipeline and transformation traits.
//!
//! In Tokel, a transformer is a component that maps an input token stream
//! to an output token stream. Transformers are applied sequentially to
//! the fully resolved output of an expansion block (`[< ... >]`).
//!
//! This module defines the core [`Transformer`] trait that all built-in (and potentially
//! custom) token manipulators must implement, along with standard utility transformers
//! like [`Identity`].

use std::fmt;

use proc_macro2::TokenStream;

use syn::parse::Nothing;

/// A transformer in a pipeline.
///
/// A transformer takes a fully resolved `TokenStream` from the preceding step in an
/// expansion block and mutates it into a new `TokenStream`.
///
/// Implementors of this trait define the behavior of individual operations (e.g.,
/// `:case`, `:concatenate`, `:reverse`) in a `Pipeline`. Transformers can maintain
/// internal state, allowing for complex, stateful token generation across a session.
pub trait Transformer {
    /// Transforms an input [token stream] into an output one.
    ///
    /// It receives the `input` tokens and the raw `argument` token stream,
    /// returning the modified tokens or a `syn::Error` if the transformation
    /// is invalid.
    ///
    /// [token stream]: TokenStream
    fn transform(&mut self, input: TokenStream, argument: TokenStream) -> syn::Result<TokenStream>;
}

/// An identity [`Transformer`].
///
/// This transformer acts as a no-op, passing the input token stream through
/// without any mutations. It takes no arguments.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity;

impl Transformer for Identity {
    #[inline]
    fn transform(&mut self, input: TokenStream, argument: TokenStream) -> syn::Result<TokenStream> {
        let _: Nothing = syn::parse2(argument)?;

        Ok(input)
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Identity")
    }
}
