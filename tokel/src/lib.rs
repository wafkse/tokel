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
#![cfg_attr(not(feature = "engine"), no_std)]
//! # `tokel`
//!
//! `tokel` is a facade crate that re-exports both `tokel-engine` and `tokel-derive`
//! when the appropriate crate features have been selected.
//!
//! * `tokel-engine`: enabled via the `engine` feature.
//! * `tokel-derive`: enabled via the `derive` feature (default). The `tokel::stream!` and `tokel::item!`
//!   macros are re-exported at the crate root for usability.
//!
//! This crate is `no_std` by default when the `engine` feature is disabled.
//!
//! ---
//!
//! *The following is the main `tokel` workspace documentation:*
//!
#![doc = include_str!("../../README.md")]

#[cfg(feature = "engine")]
pub mod engine {
    //! A module to re-export all public items from the `tokel-engine` crate.
    //!
    //! For further information, see the [crate-level documentation] of the relevant crate.
    //!
    //! [crate-level documentation]: tokel_engine

    pub use tokel_engine::*;
}

#[cfg(feature = "derive")]
pub mod derive {
    //! A module to re-export all public items from the `tokel-derive` crate.
    //!
    //! For further information, see the [crate-level documentation] of the relevant crate.
    //!
    //! [crate-level documentation]: tokel_derive

    pub use tokel_derive::*;
}

#[cfg(feature = "derive")]
#[doc(inline)]
pub use crate::derive::{attribute, stream};
