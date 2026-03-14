//! The evaluation context for Tokel macro expansions.
//!
//! A [`Session`] represents a single, temporary run of the Tokel engine. It holds the
//! [`Registry`] of available transformers and maintains their internal state throughout
//! the bottom-up evaluation of an abstract syntax tree.
//!
//! Because transformers can be stateful (e.g., counters or accumulators), creating a
//! fresh [`Session`] for each macro invocation ensures that state is cleanly isolated
//! and automatically dropped when the expansion is complete.

use crate::transform::Registry;

/// The execution context for a Tokel expansion.
///
/// This struct holds the active [`Registry`] and is responsible for evaluating
/// `TokelStream`s. By default, it is pre-loaded with Tokel's standard built-in
/// transformers, but it can be customized or strictly sandboxed as needed.
#[derive(Debug, Default)]
pub struct Session(Registry);

impl Session {
    /// Creates a new [`Session`] populated with the default [`Registry`].
    ///
    /// This includes all standard Tokel transformers (e.g., `identity`, `case`).
    /// This is the primary constructor used by standard `tokel::stream!` invocations.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::new_with(Registry::default())
    }

    /// Creates a [`Session`] using a specific, pre-configured [`Registry`].
    ///
    /// This is useful if you want to initialize a registry, heavily customize it,
    /// and then pass it into the session for execution.
    #[inline]
    #[must_use]
    pub fn new_with(registry: Registry) -> Self {
        Self(registry)
    }

    /// Creates a completely blank [`Session`] with an empty [`Registry`].
    ///
    /// This session will have no built-in transformers available. It is useful
    /// for creating strictly controlled, domain-specific token manipulation APIs
    /// where users are only allowed to use custom transformers you explicitly provide.
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self(Registry::empty())
    }

    /// Borrows the active [`Registry`] for this [`Session`].
    #[inline]
    #[must_use]
    pub const fn registry(&self) -> &Registry {
        let &Self(ref target_value) = self;

        target_value
    }

    /// Mutably borrows the active [`Registry`] for this [`Session`].
    ///
    /// This allows you to register new custom transformers or extract state from
    /// existing ones during or after the session's lifecycle.
    #[inline]
    #[must_use]
    pub const fn registry_mut(&mut self) -> &mut Registry {
        let &mut Self(ref mut target_value) = self;

        target_value
    }
}
