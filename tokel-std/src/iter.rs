//! Iteration-related Tokel [`Transformer`]s.

use std::{iter, ops::Range};

use proc_macro2::{Literal, TokenStream, TokenTree};
use syn::{
    LitInt, Token,
    parse::{Nothing, Parse, ParseStream},
};
use tokel_engine::prelude::{Registry, Transformer};

/// A transformer that reverses the sequence of the token trees in the stream.
///
/// # Example
/// `[< a b c >]:reverse` -> `c b a`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reverse;

impl Transformer for Reverse {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let _: Nothing = syn::parse2(argument)?;

        let mut tokens: Vec<_> = input.into_iter().collect();

        tokens.reverse();

        Ok(tokens.into_iter().collect())
    }
}

/// Inserts the argument between every token tree in the input.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Intersperse;

impl Transformer for Intersperse {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let mut output = TokenStream::new();
        let mut iter = input.into_iter().peekable();

        while let Some(tree) = iter.next() {
            output.extend(iter::once(tree));

            if iter.peek().is_some() {
                output.extend(argument.clone());
            }
        }

        Ok(output)
    }
}

/// Prepends the argument to the start of the stream.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PushLeft;

impl Transformer for PushLeft {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let mut output = argument;

        output.extend(input);

        Ok(output)
    }
}

/// Removes the last token tree from the stream (useful for trailing commas).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PopRight;

impl Transformer for PopRight {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let _: Nothing = syn::parse2(argument)?;

        let mut tokens: Vec<_> = input.into_iter().collect();

        tokens.pop(); // Discard the last token

        Ok(tokens.into_iter().collect())
    }
}

/// Appends the argument to the end of the stream.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PushRight;

impl Transformer for PushRight {
    fn transform(
        &mut self,
        mut input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        input.extend(argument);
        Ok(input)
    }
}

/// Removes the first token tree from the stream.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PopLeft;

impl Transformer for PopLeft {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let _: Nothing = syn::parse2(argument)?;

        let mut iter = input.into_iter();
        iter.next(); // Discard the first token

        Ok(iter.collect())
    }
}

/// Keeps only the first `N` tokens from the stream.
///
/// The argument must be a valid integer literal, e.g., `[[ 3 ]]`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Take;

impl Transformer for Take {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let lit: syn::LitInt = syn::parse2(argument)?;
        let n: usize = lit.base10_parse()?;

        Ok(input.into_iter().take(n).collect())
    }
}

/// Discards the first `N` tokens from the stream.
///
/// The argument must be a valid integer literal, e.g., `[[ 2 ]]`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Skip;

impl Transformer for Skip {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let lit: syn::LitInt = syn::parse2(argument)?;

        Ok(input.into_iter().skip(lit.base10_parse()?).collect())
    }
}

/// Duplicates the input stream `N` times.
///
/// The argument must be a valid integer literal, e.g., `[[ 5 ]]`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Repeat;

impl Transformer for Repeat {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let lit: syn::LitInt = syn::parse2(argument)?;

        let n: usize = lit.base10_parse()?;

        let mut output = TokenStream::new();

        for _ in 0..n {
            output.extend(input.clone());
        }

        Ok(output)
    }
}

/// Emits the number of token trees present in the stream.
///
/// E.g., `[< a b c >]:measure` -> `3`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Count;

impl Transformer for Count {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        let _: Nothing = syn::parse2(argument)?;

        let target_count = input.into_iter().count();

        let target_literal = Literal::usize_unsuffixed(target_count);

        Ok(quote::quote!(#target_literal))
    }
}

/// Generates a sequence of integers based on a provided range.
///
/// The input stream is ignored. The argument must be a valid Rust range literal.
/// Supported formats: `[[ 0..5 ]]` or `[[ 1..=10 ]]`.
///
/// # Example
/// `[< >]:sequence[[ 1..=3 ]]` -> `1 2 3`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sequence;

impl Transformer for Sequence {
    fn transform(&mut self, input: TokenStream, argument: TokenStream) -> syn::Result<TokenStream> {
        struct Argument(Range<i32>);

        impl Parse for Argument {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let start: LitInt = input.parse()?;

                let _: Token![..] = input.parse()?;

                let end: LitInt = input.parse()?;

                Ok(Self(start.base10_parse()?..end.base10_parse()?))
            }
        }

        let _: Nothing = syn::parse2(input)?;

        let Argument(target_range) = syn::parse2(argument)?;

        let mut output = TokenStream::new();

        for target_value in target_range {
            output.extend(iter::once(TokenTree::Literal(Literal::i32_unsuffixed(
                target_value,
            ))));
        }

        Ok(output)
    }
}

/// Inserts all `iter`-related [`Transformer`]s into the specified [`Registry`].
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
