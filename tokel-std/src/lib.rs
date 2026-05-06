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
//! # `tokel-std`
//!
//! A standard library of minimal, composable [`Transformer`] implementations for the Tokel engine.
//!
//! ## Available Transformers
//!
//! | Transformer       | Description | Example |
//! |-------------------|-------------|---------|
//! | [`Reverse`]       | Reverses the sequence of token trees. | `[< a b c >]:reverse` -> `c b a` |
//! | [`Intersperse`]   | Inserts a token tree between each input token tree. | `[< a b c >]:intersperse[[,]]` -> `a , b , c` |
//! | [`PushLeft`]      | Prepends a `TokenStream` to the input. | `[< b c >]:push_left[[a]]` -> `a b c` |
//! | [`PushRight`]     | Appends a `TokenStream` to the input. | `[< a b >]:push_right[[c]]` -> `a b c` |
//! | [`PopLeft`]       | Removes the first token tree from the input. | `[< a b c >]:pop_left` -> `b c` |
//! | [`PopRight`]      | Removes the last token tree from the input. | `[< a b c >]:pop_right` -> `a b` |
//! | [`Take`]          | Keeps the first N token trees. (Argument: `syn::LitInt`). | `[< a b c d >]:take[[2]]` -> `a b` |
//! | [`Skip`]          | Discards the first N token trees. (Argument: `syn::LitInt`). | `[< a b c d >]:skip[[2]]` -> `c d` |
//! | [`Repeat`]        | Repeats the input N times. (Argument: `syn::LitInt`). | `[< a b >]:repeat[[3]]` -> `a b a b a b` |
//! | [`Count`]         | Returns the number of input token trees as an integer literal. | `[< a b c >]:count` -> `3` |
//! | [`Sequence`]      | Yields an integer sequence. (Argument: `[[ start..end ]]`). | `[< >]:sequence[[1..=3]]` -> `1 2 3` |
//! | [`Enumerate`]     | Stateful generator yielding an incrementing `u32` literal. | `[< >]:enumerate` -> `0` |
//! | [`Concatenate`]   | Concatenates text representations of tokens into a single `Ident` or string literal. | `[< hello _ world >]:concatenate` -> `hello_world` |
//! | [`ToString`]      | Stringifies arbitrary tokens into string literals. | `[< hello _ world >]:to_string` -> `"hello" "_" "world"` |
//! | [`Case`]          | Converts identifiers and strings to a target [`CaseStyle`]. | `[< hello_world >]:case[[pascal]]` -> `HelloWorld` |
//! | [`Flatten`]       | Recursively removes all group boundaries, flattening into a 1D stream. | `[< (()) [[Hello]] ([{ AA }]) >]:flatten` -> `Hello AA`. This is useful for getting rid of invisible `None`-delimited groups.  |
//! | [`Encapsulate`]   | Wraps the input inside the given group argument delimiter. | `[< Hello >]:encapsulate[[()]]` -> `(Hello)` |
//!
//! ## Notes
//!
//! * **Arguments:** Transformers can accept other transformer expressions as arguments. Inner expressions are evaluated first.
//! * **State:** Stateful transformers (like [`Enumerate`]) keep their state for the lifetime of the registered instance.
//! * **Details:** See the module-level documentation ([`iter`], [`string`], [`structure`], [`state`]) for specific argument types and behaviors.
//!
//! [`Transformer`]: tokel_engine::prelude::Transformer
//! [`Reverse`]: iter::Reverse
//! [`Intersperse`]: iter::Intersperse
//! [`PushLeft`]: iter::PushLeft
//! [`PushRight`]: iter::PushRight
//! [`PopLeft`]: iter::PopLeft
//! [`PopRight`]: iter::PopRight
//! [`Take`]: iter::Take
//! [`Skip`]: iter::Skip
//! [`Repeat`]: iter::Repeat
//! [`Count`]: iter::Count
//! [`Sequence`]: iter::Sequence
//! [`Enumerate`]: state::Enumerate
//! [`Concatenate`]: string::Concatenate
//! [`ToString`]: string::ToString
//! [`Case`]: string::Case
//! [`CaseStyle`]: string::CaseStyle
//! [`Flatten`]: structure::Flatten
//! [`Encapsulate`]: structure::Encapsulate
//! [`iter`]: crate::iter
//! [`string`]: crate::string
//! [`structure`]: crate::structure
//! [`state`]: crate::state
//!
//! ---
//!
//! *The following is the main `tokel` workspace documentation:*
#![doc = include_str!("../README.md")]

use tokel_engine::prelude::{Registry, Transformer};

pub mod string;

pub mod iter;

pub mod structure;

pub mod state;

/// Registers all standard `Transformer`s into the provided `Registry`.
///
/// This inserts the default transformers from the `string`, `iter`, `structure`,
/// and `state` modules.
///
/// # Errors
///
/// Returns an error if a transformer with the same name is already in the `Registry`.
/// If this happens, any transformers that were successfully registered before the error
/// will remain in the registry.
#[inline]
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn Transformer>> {
    string::register(registry)?;

    iter::register(registry)?;

    structure::register(registry)?;

    state::register(registry)?;

    Ok(())
}
