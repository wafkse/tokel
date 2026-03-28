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
//! # `tokel-engine`
//!
//! The engine crate for *Tokel*.
//!
//! ---
//!
//! *The following is the main `tokel` workspace documentation:*
//!
#![doc = include_str!("../README.md")]

pub mod transform;

pub mod syntax;

pub mod session;

pub mod expand;

pub mod prelude {
    //! The prelude module of the `tokel-engine` crate.

    pub use crate::transform::{Pass, Registry, Transformer};

    pub use crate::expand::Expand;

    pub use crate::session::Session;

    pub use crate::syntax::TokelStream;
}
