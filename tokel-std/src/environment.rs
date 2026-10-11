//! Compile-time environment inputs for Tokel transformer pipelines.

use std::env;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Error, LitStr, parse::Nothing};
use tokel_engine::prelude::{Pass, Registry, Transformer};

/// Read an environment variable into a string literal during expansion.
///
/// The input must be empty. The argument is the variable name as a string literal.
/// The resulting literal inherits the argument's span and can be passed through
/// [`Unstringify`](crate::string::Unstringify) to obtain Rust tokens.
///
/// This reads the environment of the process running the transformer. For Cargo
/// rebuild tracking, retain an `env!` or `options_env!` reference to the same variable in the caller.
///
/// # Example
///
/// `[< >]:environment[["CARGO_PKG_NAME"]]` yields the package name as a string literal.
/// `[< >]:environment[["CARGO_PKG_VERSION_MAJOR"]]:unstringify` yields an integer literal.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Environment;

impl Pass for Environment {
    type Argument = LitStr;

    fn through(&mut self, input: TokenStream, name: Self::Argument) -> syn::Result<TokenStream> {
        let _: Nothing = syn::parse2(input)?;

        let value = env::var(name.value()).map_err(|error| {
            Error::new(
                name.span(),
                format!(
                    "cannot read environment variable `{}`: {error}",
                    name.value()
                ),
            )
        })?;

        Ok(LitStr::new(&value, name.span()).into_token_stream())
    }
}

/// Register the `environment` transformer.
///
/// # Errors
///
/// Returns the transformer if its name is already registered.
#[inline]
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn Transformer>> {
    registry
        .try_insert("environment", Environment)
        .map_err(Box::new)
        .map_err(|transformer| transformer as Box<dyn Transformer>)?;

    Ok(())
}
