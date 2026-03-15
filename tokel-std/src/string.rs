//! String and text-manipulation Tokel [`Transformer`]s.

use proc_macro2::{Ident, Span, TokenStream};

use syn::parse::Nothing;

use tokel_engine::prelude::{Registry, Transformer};

/// A transformer that concatenates all input tokens into a single identifier.
///
/// It ignores standard spacing and simply glues the string representations
/// of the tokens together.
///
/// # Example
/// `[< hello _ world >]:concatenate` -> `hello_world`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Concatenate;

impl Transformer for Concatenate {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        // Concatenate takes no arguments, so we enforce that the `[[...]]` is empty.
        let _: Nothing = syn::parse2(argument)?;

        // If the input is completely empty, just return empty.
        if input.is_empty() {
            return Ok(input);
        }

        let mut concatenated_string = String::new();
        let mut first_span = Span::call_site();
        let mut is_first = true;

        for tree in input {
            if is_first {
                first_span = tree.span();

                is_first = false;
            }

            // `to_string()` on a TokenTree strips `r#` from idents and handles raw strings nicely.
            concatenated_string.push_str(&tree.to_string());
        }

        // We must ensure the resulting string is a valid Rust identifier.
        // `syn::Ident::new` will panic if the string is not a valid ident (e.g. if it starts with a number).
        // To be safe, we try to parse it. If it fails, we return a syn::Error.
        let parsed_ident = syn::parse_str::<Ident>(&concatenated_string).map_err(|_| {
            syn::Error::new(
                first_span,
                format!("concatenated string `{concatenated_string}` is not a valid identifier"),
            )
        })?;

        Ok(quote::quote_spanned!(first_span=> #parsed_ident))
    }
}

/// Inserts all `string`-related [`Transformer`]s into the specified [`Registry`].
///
/// # Errors
///
/// This will fail if at least one standard `string`-related [`Transformer`] is already present by-name in the [`Registry`].
///
/// On failure, there is no guarantee that other non-colliding transformers have not been registered.
#[inline]
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn Transformer>> {
    registry
        .try_insert("concatenate", Concatenate)
        .map_err(Box::new)
        .map_err(|target_value| target_value as Box<dyn Transformer>)?;

    Ok(())
}
