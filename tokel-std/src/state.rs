//! Stateful generation [`Transformer`]s.
//!
//! # Available Transformers
//!!
//! | Transformer   | Argument Type                 | Description |
//! |---------------|-------------------------------|-------------|
//! | [`Enumerate`] | [`syn::parse::Nothing`]       | Yields an incrementing integer literal on each invocation (stateful). |
//!
//! # Argument Types
//!
//! - [`syn::parse::Nothing`]: No argument required.
//!
//! # Examples
//!
//! * `[< >]:enumerate` ->`0`
//! * Calling the same transformer again advances its state: `[< >]:enumerate` ->`1`
//! * Use `enumerate` inside another transformer's argument:
//!   * `[< b c >]:push_left[[[< >]:enumerate]]` ->`0 b c`
//!   * `[< b c >]:push_right[[[< >]:enumerate]]` ->`b c 1`

use proc_macro2::{Literal, TokenStream};

use quote::ToTokens;
use tokel_engine::prelude::{Pass, Registry, Transformer};

/// Yields an incrementing integer literal every time it is called.
///
/// # Argument
///
/// This takes no argument.
///
/// # Examples
///
/// * `[< >]:enumerate` ->`0`
/// * As an argument to another transformer: `[< a b >]:push_left[[[< >]:enumerate]]` ->`0 a b`
///
/// # Remarks
///
/// The integer is internally an [`u32`], and is incremented with wrapping arithmetic.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enumerate(u32);

impl Pass for Enumerate {
    type Argument = syn::parse::Nothing;

    fn through(&mut self, _: TokenStream, _: Self::Argument) -> syn::Result<TokenStream> {
        let Self(enumerate_counter) = self;

        let current_value;
        (current_value, *enumerate_counter) =
            (*enumerate_counter, enumerate_counter.wrapping_add(1));

        Ok(Literal::u32_unsuffixed(current_value).into_token_stream())
    }
}

/// Inserts all `state`-related [`Transformer`]s into the specified [`Registry`].
///
/// # Errors
///
/// This will fail if at least one standard `state`-related [`Transformer`] is already present by-name in the [`Registry`].
///
/// On failure, there is no guarantee that other non-colliding transformers have not been registered.
#[inline]
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn Transformer>> {
    registry
        .try_insert("enumerate", Enumerate::default())
        .map_err(Box::new)
        .map_err(|target_value| target_value as Box<dyn Transformer>)?;

    Ok(())
}
