//! The transformer pipeline and transformation traits.
//!
//! In Tokel, a transformer is a component that maps an input token stream
//! to an output token stream. Transformers are applied sequentially to
//! the fully resolved output of an expansion block (`[< ... >]`).
//!
//! This module defines the core [`Transformer`] trait that all built-in (and potentially
//! custom) token manipulators must implement, along with standard utility transformers
//! like [`Identity`].

use std::{
    any::Any,
    borrow::Cow,
    collections::hash_map::Entry,
    fmt,
    ops::{Deref, DerefMut},
    ptr,
};

use ahash::AHashMap;

use proc_macro2::TokenStream;

use syn::parse::Nothing;

/// A transformer in a pipeline.
///
/// A transformer takes a fully resolved `TokenStream` from the preceding step in an
/// expansion block and mutates it into a new `TokenStream`.
///
/// Implementors of this trait define the behavior of individual operations (e.g.,
/// `:case`, `:concatenate`, `:reverse`) in a `Pipeline`. Transformers can maintain
/// internal state, allowing for complex, stateful token generation across a session.
pub trait Transformer: Any {
    /// Transforms an input [token stream] into an output one.
    ///
    /// It receives the `input` tokens and the raw `argument` token stream,
    /// returning the modified tokens or a `syn::Error` if the transformation
    /// is invalid.
    ///
    /// [token stream]: TokenStream
    ///
    /// # Errors
    ///
    /// The failure mode of this associated function is implementation-dependent.
    fn transform(&mut self, input: TokenStream, argument: TokenStream) -> syn::Result<TokenStream>;
}

/// A centralized registry of [`Transformer`] of distinct types.
///
/// This maps an unique name to a boxed [`Transformer`] trait object.
///
/// # Identifier Resolution
///
/// When the Tokel engine encounters a transformer in a pipeline (e.g., `:case`),
/// it converts the parsed `syn::Ident` to a string using exactly its written case.
/// Raw identifiers (e.g., `r#case`) have their `r#` prefix stripped before lookup.
///
/// Therefore, transformer names registered here should typically be exact,
/// `snake_case` string literals matching the intended syntax.
pub struct Registry(AHashMap<Cow<'static, str>, Box<dyn Transformer>>);

impl Registry {
    /// Creates an empty [`Registry`].
    #[must_use]
    pub fn empty() -> Self {
        Self(AHashMap::new())
    }

    /// Inserts a [`Transformer`] into the [`Registry`].
    ///
    /// If a [`Transformer`] was already registered with an identical name, it is yielded back.
    #[inline]
    pub fn insert<S, T>(&mut self, name: S, transformer: T) -> Option<Box<dyn Transformer>>
    where
        S: Into<Cow<'static, str>>,
        T: Transformer,
    {
        let &mut Self(ref mut map) = self;

        map.insert(S::into(name), Box::new(transformer))
    }

    /// Attempts to insert a [`Transformer`] into the [`Registry`].
    ///
    /// # Errors
    ///
    /// This will fail if another [`Transformer`] with the same name has already been registered.
    #[inline]
    pub fn try_insert<S, T>(
        &mut self,
        name: S,
        transformer: T,
    ) -> Result<&mut Box<dyn Transformer>, T>
    where
        S: Into<Cow<'static, str>>,
        T: Transformer,
    {
        let &mut Self(ref mut map) = self;

        let key = S::into(name);

        match map.entry(key) {
            Entry::Occupied(..) => Err(transformer),
            Entry::Vacant(vacant_entry) => Ok(vacant_entry.insert(Box::new(transformer))),
        }
    }
}

impl Deref for Registry {
    type Target = AHashMap<Cow<'static, str>, Box<dyn Transformer>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        let Self(target_value) = self;

        target_value
    }
}

impl DerefMut for Registry {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self(target_value) = self;

        target_value
    }
}

impl Default for Registry {
    fn default() -> Self {
        let mut target_value = Self::empty();

        let _ = target_value.insert("identity", Identity);

        target_value
    }
}

impl fmt::Debug for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(target_value) = self;

        let mut target_state = f.debug_map();

        for (entry_key, entry_transformer) in target_value {
            // NOTE: We don't know anything about the Transformer but its address, so let's mention it.
            let target_value = &format_args!(
                "<transformer at {:#x}>",
                ptr::from_ref(entry_transformer).addr()
            );

            target_state.entry(entry_key, target_value);
        }

        target_state.finish()
    }
}

/// An identity [`Transformer`].
///
/// This transformer acts as a no-op, passing the input token stream through
/// without any mutations. It takes no arguments.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity;

impl Transformer for Identity {
    #[inline]
    fn transform(&mut self, input: TokenStream, argument: TokenStream) -> syn::Result<TokenStream> {
        let _: Nothing = syn::parse2(argument)?;

        Ok(input)
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Identity")
    }
}
