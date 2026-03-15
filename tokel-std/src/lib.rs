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
//! `tokel-std` is a crate exposing a set of minimal, easily-composable transformer implementations that are deemed standard.
//!
//! ---
//!
//! *The following is the main `tokel` workspace documentation:*
//!
#![doc = include_str!("../README.md")]

use tokel_engine::prelude::{Registry, Transformer};

pub mod string;

pub mod iter;

pub mod structure;

pub mod state;

/// Register all the standard [`Transformer`] implementations
///
/// # Errors
///
/// This will fail if at least one standard [`Transformer`] is already present by-name in the [`Registry`].
///
/// On failure, there is no guarantee that other non-colliding transformers have not been registered.
#[inline]
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn Transformer>> {
    string::register(registry)?;

    iter::register(registry)?;

    structure::register(registry)?;

    state::register(registry)?;

    Ok(())
}
