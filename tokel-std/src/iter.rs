//! Iteration-related Tokel [`Transformer`]s.
//!
//! # Available Transformers
//!
//! | Transformer         | Argument Type         | Description |
//! |---------------------|----------------------|-------------|
//! | [`Reverse`]         | [`syn::parse::Nothing`]     | Reverses the sequence of token trees. |
//! | [`Intersperse`]     | [`TokenTree`]        | Inserts a token-tree between each token-tree in the input. |
//! | [`PushLeft`]        | [`TokenStream`]      | Prepends a token stream to the input. |
//! | [`PushRight`]       | [`TokenStream`]      | Appends a token stream to the input. |
//! | [`PopLeft`]         | [`syn::parse::Nothing`]     | Removes the first token tree from the stream. |
//! | [`PopRight`]        | [`syn::parse::Nothing`]     | Removes the last token tree from the stream. |
//! | [`Take`]            | [`syn::LitInt`]      | Keeps only the first `N` tokens from the stream. |
//! | [`Skip`]            | [`syn::LitInt`]      | Discards the first `N` tokens from the stream. |
//! | [`Repeat`]          | [`syn::LitInt`]      | Duplicates the input stream `N` times. |
//! | [`Count`]           | [`syn::parse::Nothing`]     | Emits the number of token trees present in the stream. |
//! | [`Sequence`]        | [`SequenceRange`]    | Generates a sequence of integers based on a provided range. |
//!
//! # Argument Types
//!
//! - [`syn::parse::Nothing`]: No argument required.
//! - [`TokenTree`]: A single token tree (e.g., a punctuation, identifier, etc.).
//! - [`TokenStream`]: A sequence of token trees.
//! - [`syn::LitInt`]: An integer literal (e.g., `[[ 3 ]]`).
//! - [`SequenceRange`]: A Rust range literal (e.g., `[[ 1..=3 ]]`).
//!
//! # Examples
//!
//! - `[< a b c >]:reverse` ->`c b a`
//! - `[< a b c >]:intersperse[[,]]` ->`a , b , c`
//! - `[< b c >]:push_left[[a]]` ->`a b c`
//! - `[< a b c >]:pop_right` ->`a b`
//! - `[< a b c >]:take[[2]]` ->`a b`
//! - `[< a b c >]:skip[[1]]` ->`b c`
//! - `[< a b >]:repeat[[3]]` ->`a b a b a b`
//! - `[< a b c >]:count` ->`3`
//! - `[< >]:sequence[[ 1..=3 ]]` ->`1 2 3`
//!
//! * Transformers can be nested or composed by evaluating other transformers inside arguments:
//!
//! - `[< a b c >]:intersperse[[[< x y >]:reverse]]` ->`a y x b y x c`
//! - `[< a b c >]:push_left[[[< x y >]:reverse]]` ->`y x a b c`
//! - `[< a b c >]:push_right[[[< d e >]:take[[1]]]]` ->`a b c d`
//! - `[< 1 2 3 >]:take[[[< 2 1 >]:reverse:take[[1]]]]` ->`1 2`
//! - `[< a b c >]:repeat[[[< 2 1 >]:count]]` ->`a b c a b c`
//! - `[< >]:sequence[[[< 1 4 >]:reverse:take[[1]]..4]]` ->`3 4 5 6`

use std::ops::{Range, RangeInclusive};

use proc_macro2::{Literal, TokenStream, TokenTree};

use quote::ToTokens;

use syn::{
    Token,
    parse::{Parse, ParseStream},
};

use tokel_engine::prelude::{Pass, Registry, Transformer};

/// Reverses the sequence of token trees in the stream.
///
/// # Arguments
///
/// This does not take any argument.
///
/// # Example
///
/// `[< a b c >]:reverse` ->`c b a`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reverse;

impl Pass for Reverse {
    type Argument = syn::parse::Nothing;

    fn through(&mut self, input: TokenStream, _: Self::Argument) -> syn::Result<TokenStream> {
        Ok(input
            .into_iter()
            .collect::<Vec<TokenTree>>()
            .into_iter()
            .rev()
            .collect::<TokenStream>())
    }
}

/// Inserts the provided token-tree in between each token-tree in the input.
///
/// # Arguments
///
/// This takes a singular [`TokenTree`] as argument.
///
/// # Example
///
/// `[< a b c >]:intersperse[[,]]` ->`a , b , c`
///
/// *NOTE*: Most tokens will be preserved *verbatim*, including any span-related information.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Intersperse;

impl Pass for Intersperse {
    type Argument = TokenTree;

    fn through(
        &mut self,
        input: TokenStream,
        intersperse_tree: Self::Argument,
    ) -> syn::Result<TokenStream> {
        let mut target_output = TokenStream::new();

        let mut target_iter = input.into_iter().peekable();

        while let Some(target_tree) = target_iter.next() {
            let target_list = [
                Some(target_tree),
                if target_iter.peek().is_some() {
                    let intersperse_tree = intersperse_tree.clone();

                    Some(intersperse_tree)
                } else {
                    None
                },
            ];

            target_output.extend(target_list.into_iter().flatten());
        }

        Ok(target_output)
    }
}

/// Pushes the provided token stream to the start (*left*) of the input token stream.
///
/// # Arguments
///
/// This takes a singular [`TokenStream`] as argument.
///
/// # Example
///
/// `[< b c >]:push_left[[a]]` ->`a b c`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PushLeft;

impl Pass for PushLeft {
    type Argument = TokenStream;

    fn through(&mut self, input: TokenStream, left: Self::Argument) -> syn::Result<TokenStream> {
        Ok(left.into_iter().chain(input).collect::<TokenStream>())
    }
}

/// Removes the last token tree from the stream (useful for trailing commas or other garbage).
///
/// # Arguments
///
/// This does not take any argument.
///
/// # Example
///
/// `[< a b c >]:pop_right` ->`a b`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PopRight;

impl Pass for PopRight {
    type Argument = syn::parse::Nothing;

    fn through(&mut self, input: TokenStream, _: Self::Argument) -> syn::Result<TokenStream> {
        let mut target_list = input.into_iter().collect::<Vec<TokenTree>>();

        let _ = target_list.pop();

        Ok(target_list.into_iter().collect::<TokenStream>())
    }
}

/// Appends the provided token stream to the end (*right*) of the input token stream.
///
/// # Arguments
///
/// This takes a singular [`TokenStream`] as argument.
///
/// # Example
///
/// `[< a b >]:push_right[[c]]` ->`a b c`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PushRight;

impl Pass for PushRight {
    type Argument = TokenStream;

    fn through(&mut self, input: TokenStream, right: Self::Argument) -> syn::Result<TokenStream> {
        Ok(input.into_iter().chain(right).collect::<TokenStream>())
    }
}

/// Removes the first token tree from the stream.
///
/// # Arguments
///
/// This does not take any argument.
///
/// # Example
///
/// * `[< a b c >]:pop_left` ->`b c`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PopLeft;

impl Pass for PopLeft {
    type Argument = syn::parse::Nothing;

    fn through(&mut self, input: TokenStream, _: Self::Argument) -> syn::Result<TokenStream> {
        Ok(input.into_iter().skip(1).collect::<TokenStream>())
    }
}

/// Keeps only the first `N` tokens from the stream.
///
/// # Arguments
///
/// This takes a single [`syn::LitInt`] as argument (e.g., `[[ 3 ]]`).
///
/// # Example
///
/// * `[< a b c d >]:take[[2]]` ->`a b`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Take;

impl Pass for Take {
    type Argument = syn::LitInt;

    fn through(&mut self, input: TokenStream, count: Self::Argument) -> syn::Result<TokenStream> {
        let take_count = count.base10_parse()?;

        Ok(input.into_iter().take(take_count).collect())
    }
}

/// Discards the first `N` tokens from the stream.
///
/// # Arguments
///
/// This takes a single [`syn::LitInt`] as argument (e.g., `[[ 2 ]]`).
///
/// # Example
///
/// `[< a b c d >]:skip[[2]]` ->`c d`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Skip;

impl Pass for Skip {
    type Argument = syn::LitInt;

    fn through(&mut self, input: TokenStream, count: Self::Argument) -> syn::Result<TokenStream> {
        let skip_count = count.base10_parse()?;

        Ok(input.into_iter().skip(skip_count).collect())
    }
}

/// Duplicates the input stream `N` times.
///
/// # Arguments
///
/// This takes a single [`syn::LitInt`] as argument (e.g., `[[ 3 ]]`).
///
/// # Example
///
/// * `[< a b >]:repeat[[3]]` ->`a b a b a b`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Repeat;

impl Pass for Repeat {
    type Argument = syn::LitInt;

    fn through(
        &mut self,
        input: TokenStream,
        target_count: Self::Argument,
    ) -> syn::Result<TokenStream> {
        let target_list = input.into_iter().collect::<Vec<TokenTree>>();

        let take_count = target_list.len() * target_count.base10_parse::<usize>()?;

        Ok(target_list
            .iter()
            .cycle()
            .take(take_count)
            .cloned()
            .collect::<TokenStream>())
    }
}

/// Emits the number of token trees present in the stream.
///
/// # Arguments
///
/// This does not take any argument.
///
/// # Example
///
/// * `[< a b c >]:count` ->`3`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Count;

impl Pass for Count {
    type Argument = syn::parse::Nothing;

    fn through(&mut self, input: TokenStream, _: Self::Argument) -> syn::Result<TokenStream> {
        Ok(Literal::usize_unsuffixed(input.into_iter().count()).to_token_stream())
    }
}

/// The range argument of a [`Sequence`] pass.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SequenceRange {
    /// An inclusive range.
    Inclusive(RangeInclusive<i32>),

    /// A non-inclusive range.
    NonInclusive(Range<i32>),
}

impl Parse for SequenceRange {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let start: syn::LitInt = input.parse()?;

        if input.peek(Token![..=]) {
            let _: Token![..=] = input.parse()?;

            let end: syn::LitInt = input.parse()?;

            Ok(Self::Inclusive(start.base10_parse()?..=end.base10_parse()?))
        } else {
            let _: Token![..] = input.parse()?;

            let end: syn::LitInt = input.parse()?;

            Ok(Self::NonInclusive(
                start.base10_parse()?..end.base10_parse()?,
            ))
        }
    }
}

/// Generates a sequence of integers based on a provided range.
///
/// The input stream is ignored.
///
/// # Arguments
///
/// This takes a single [`SequenceRange`] as argument (e.g., `[[ 0..4 ]]` or `[[ 1..=3 ]]`).
///
/// # Example
///
/// * `[< >]:sequence[[ 1..4 ]]` ->`1 2 3`
/// * `[< >]:sequence[[ 1..=3 ]]` ->`1 2 3`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sequence;

impl Pass for Sequence {
    type Argument = SequenceRange;

    fn through(
        &mut self,
        input: TokenStream,
        target_range: Self::Argument,
    ) -> syn::Result<TokenStream> {
        fn fold_fn(mut target_output: TokenStream, target_literal: Literal) -> TokenStream {
            target_output.extend(target_literal.into_token_stream());

            target_output
        }

        let _: syn::parse::Nothing = syn::parse2(input)?;

        Ok(match target_range {
            SequenceRange::Inclusive(range_inclusive) => range_inclusive
                .into_iter()
                .map(Literal::i32_unsuffixed)
                .fold(TokenStream::new(), fold_fn),
            SequenceRange::NonInclusive(range) => range
                .into_iter()
                .map(Literal::i32_unsuffixed)
                .fold(TokenStream::new(), fold_fn),
        })
    }
}

/// Inserts all `iter`-related [`Transformer`]s into the specified [`Registry`].
///
/// # Errors
///
/// This will fail if at least one standard `iter`-related [`Transformer`] is already present by-name in the [`Registry`].
///
/// On failure, there is no guarantee that other non-colliding transformers have not been registered.
#[inline]
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn Transformer>> {
    registry
        .try_insert("reverse", Reverse)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("intersperse", Intersperse)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("push_left", PushLeft)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("push_right", PushRight)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("pop_left", PopLeft)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("pop_right", PopRight)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("take", Take)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("skip", Skip)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("repeat", Repeat)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("count", Count)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("sequence", Sequence)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    Ok(())
}
