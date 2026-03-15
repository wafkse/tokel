//! Stateful generation [`Transformer`]s.

use proc_macro2::{Literal, TokenStream};
use tokel_engine::prelude::{Registry, Transformer};

/// Yields an incrementing integer literal every time it is called.
///
/// The integer is internally an [`u32`], and is incremented with wrapping arithmetic.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enumerate(u32);

impl Transformer for Enumerate {
    fn transform(
        &mut self,
        _input: TokenStream,
        _argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let Self(target_count) = self;

        let current_value;

        (current_value, *target_count) = (*target_count, target_count.wrapping_add(1));

        let target_literal = Literal::u32_unsuffixed(current_value);

        Ok(quote::quote!(#target_literal))
    }
}

#[inline]
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn Transformer>> {
    registry
        .try_insert("enumerate", Enumerate::default())
        .map_err(Box::new)
        .map_err(|target_value| target_value as Box<dyn Transformer>)?;

    Ok(())
}
