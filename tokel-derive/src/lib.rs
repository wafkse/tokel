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
//! # `tokel-derive`
//!
//! This is an internal crate used to expose the standard procedural macros.
//!
//! Tokel exposes a serie of procedural macros that can be useful in distinct circumstances:
//!
//! * [`stream`]: to act upon an arbitrary token-stream
//! * [`item`]: to act upon an arbitrary Rust item
//! * [`attribute`]: to act upon an attribute attached to a Rust item.
//!
//! ---
//!
//! *The following is the main `tokel` workspace documentation:*
//!
#![doc = include_str!("../../README.md")]

use proc_macro::TokenStream;

#[proc_macro]
pub fn stream(input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn item(input: TokenStream, args: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn attribute(input: TokenStream, args: TokenStream) -> TokenStream {
    input
}
