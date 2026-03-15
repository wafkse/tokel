//! Structural boundary and grouping [`Transformer`]s.

use std::iter;

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use syn::parse::{Nothing, Parse, ParseStream};
use tokel_engine::prelude::{Registry, Transformer};

/// Recursively removes all `()`, `{}`, and `[]` boundaries, flattening into a 1D stream.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flatten;

impl Flatten {
    fn flatten_recursive(stream: TokenStream) -> TokenStream {
        let mut output = TokenStream::new();
        for tree in stream {
            match tree {
                TokenTree::Group(g) => output.extend(Self::flatten_recursive(g.stream())),
                other => output.extend(iter::once(other)),
            }
        }
        output
    }
}

impl Transformer for Flatten {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let _: Nothing = syn::parse2(argument)?;

        Ok(Self::flatten_recursive(input))
    }
}

/// Wraps the entire input stream in the delimiter type of the provided argument [`Group`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Encapsulate;

impl Transformer for Encapsulate {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        struct Argument(Delimiter);

        impl Parse for Argument {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let group = input.parse::<Group>()?;

                let _: Nothing = syn::parse2(group.stream())?;

                let _: Nothing = input.parse()?;

                Ok(Self(group.delimiter()))
            }
        }

        let Argument(delimiter) = syn::parse2(argument)?;

        let group = Group::new(delimiter, input);

        Ok(iter::once(TokenTree::Group(group)).collect())
    }
}

#[inline]
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn Transformer>> {
    registry
        .try_insert("flatten", Flatten)
        .map_err(Box::new)
        .map_err(|target_value| target_value as Box<dyn Transformer>)?;

    registry
        .try_insert("encapsulate", Encapsulate)
        .map_err(Box::new)
        .map_err(|target_value| target_value as Box<dyn Transformer>)?;

    Ok(())
}
