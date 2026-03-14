//! The transformer pipeline and transformation traits.
//!
//! In Tokel, a transformer is a pure, side-effect-free function that maps an input
//! token stream to an output token stream. Transformers are applied sequentially to
//! the fully resolved output of an expansion block (`[< ... >]`).
//!
//! This module defines the core [`Transformer`] trait that all built-in (and potentially
//! custom) token manipulators must implement, along with standard utility transformers
//! like [`Identity`].

use std::fmt;

use proc_macro2::TokenStream;

use syn::parse::{Nothing, Parse};

/// A transformer in a pipeline.
///
/// A transformer takes a fully resolved `TokenStream` from the preceding step in an
/// expansion block and mutates it into a new `TokenStream`.
///
/// Implementors of this trait define the behavior of individual operations (e.g.,
/// `:case`, `:concatenate`, `:reverse`) in a `Pipeline`.
pub trait Transformer {
    /// The argument required by this [`Transformer`].
    ///
    /// This type dictates what is parsed inside the double-bracket parameter list
    /// following the transformer's identifier (e.g., the `pascal` in `:case[[pascal]]`).
    /// If a transformer takes no arguments, this should be set to `syn::parse::Nothing`.
    type Argument: Parse;

    /// Transforms an input [token stream] into an output one.
    ///
    /// This function is strictly pure. It receives the `input` tokens and the parsed
    /// `argument`, and returns the modified tokens or a `syn::Error` if the transformation
    /// is invalid.
    ///
    /// [token stream]: TokenStream
    fn transform(input: TokenStream, argument: Self::Argument) -> Result<TokenStream, syn::Error>;
}

/// An identity [`Transformer`].
///
/// This transformer acts as a no-op, passing the input token stream through
/// without any mutations. It takes no arguments.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Identity {}

impl Transformer for Identity {
    type Argument = Nothing;

    #[inline]
    fn transform(input: TokenStream, _: Self::Argument) -> Result<TokenStream, syn::Error> {
        Ok(input)
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Since `Identity` is an empty enum, it can never be instantiated.
        // This is safe and standard for uninhabited types.
        match *self {}
    }
}
