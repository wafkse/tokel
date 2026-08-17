//! Structural boundary and grouping Tokel [`Transformer`]s.
//!
//! This module provides transformers for modifying the structural hierarchy
//! and boundaries of token streams.
//!
//! # Available Transformers
//!
//! | Transformer       | Argument Type           | Description |
//! |-------------------|-------------------------|-------------|
//! | [`Flatten`]       | [`syn::parse::Nothing`] | Recursively removes all `()`, `{}`, and `[]` boundaries, flattening into a 1D stream. |
//! | [`Encapsulate`]   | [`EncapsulateGroup`]    | Wraps the entire input stream inside the delimiter type of the provided argument. |
//!
//! # Argument Types
//!
//! * [`syn::parse::Nothing`] - No argument is required.
//! * [`EncapsulateGroup`] - A single, empty token group (e.g., `()`, `{}`, or `[]`) that specifies the target encapsulation delimiter.
//!
//! # Examples
//!
//! **Basic Usage:**
//! * `[< (()) [[Hello]] ([{ AA }]) >]:flatten` -> `Hello AA`
//! * `[< Hello >]:encapsulate[[()]]` -> `(Hello)`
//!
//! # Remarks
//!
//! * [`Flatten`] recursively traverses all nested groups within the token stream and extracts their inner contents, resulting in a single, flat, one-dimensional token sequence.
//! * [`Encapsulate`] takes the entire evaluated input stream and places it directly inside a new group matching the delimiter provided in the argument.

use std::iter;

use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};

use syn::parse::{Parse, ParseStream};

use tokel_engine::prelude::{Pass, Registry, Transformer};

/// Recursively removes all `()`, `{}`, and `[]` boundaries, flattening into a 1D stream.
///
/// # Arguments
///
/// This does not take any argument.
///
/// # Example
///
/// * `[< (()) [[Hello]] ([{ AA }]) >]:flatten` -> `Hello AA`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flatten;

impl Pass for Flatten {
    type Argument = syn::parse::Nothing;

    fn through(&mut self, input: TokenStream, _: Self::Argument) -> syn::Result<TokenStream> {
        /// A recursive function that does the flattening on token groups.
        fn flatten(target_stream: TokenStream) -> TokenStream {
            let mut target_output = TokenStream::new();

            for target_tree in target_stream {
                match target_tree {
                    TokenTree::Group(target_group) => {
                        target_output.extend(flatten(target_group.stream()));
                    }
                    flat_tree => target_output.extend(iter::once(flat_tree)),
                }
            }

            target_output
        }

        Ok(flatten(input))
    }
}

/// The argument type used for the [`Encapsulate`] pass.
///
/// This represents a single delimiter to use for the encapsulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncapsulateGroup(Delimiter);

impl Parse for EncapsulateGroup {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let group = input.parse::<Group>()?;

        let _: [syn::parse::Nothing; 2] = [syn::parse2(group.stream())?, input.parse()?];

        Ok(Self(group.delimiter()))
    }
}

/// Wraps the entire input stream in the delimiter type of the provided argument [`Group`].
///
/// # Arguments
///
/// This takes a singular, empty token group with the delimiter to encapsulate the input stream in.
///
/// # Example
///
/// * `[< Hello >]:encapsulate[[()]]` -> `(Hello)`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Encapsulate;

impl Pass for Encapsulate {
    type Argument = EncapsulateGroup;

    fn through(
        &mut self,
        stream: TokenStream,
        EncapsulateGroup(delimiter): Self::Argument,
    ) -> syn::Result<TokenStream> {
        Ok(iter::once(TokenTree::Group(Group::new(delimiter, stream))).collect::<TokenStream>())
    }
}

/// Inserts all `structure`-related [`Transformer`]s into the specified [`Registry`].
///
/// # Errors
///
/// This will fail if at least one standard `structure`-related [`Transformer`] is already present by-name in the [`Registry`].
///
/// On failure, there is no guarantee that other non-colliding transformers have not been registered.
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
