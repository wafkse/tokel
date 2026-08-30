//! String and text-manipulation Tokel [`Transformer`]s.
//!
//! This module provides transformers for modifying the textual representation
//! and casing of token streams.
//!
//! # Available Transformers
//!
//! | Transformer     | Argument Type           | Description |
//! |-----------------|-------------------------|-------------|
//! | [`Concatenate`] | [`syn::parse::Nothing`] | Concatenates all input tokens into a single identifier or group. |
//! | [`ToString`] | [`syn::parse::Nothing`] | Concatenates all input tokens into a single identifier or group. |
//! | [`Case`]        | [`CaseStyle`]           | Converts identifiers and string-like tokens to a target case style. |
//!
//! # Argument Types
//!
//! * [`syn::parse::Nothing`] - No argument is required.
//! * [`CaseStyle`] - A specific case formatting rule: `pascal`, `camel`, or `snake`.
//!
//! # Examples
//!
//! **Basic Usage:**
//! * `[< hello _ world >]:concatenate` -> `hello_world`
//! * `[< hello _ world >]:case[[pascal]]` -> `Hello _ World`
//! * `[< some_value >]:case[[camel]]` -> `someValue`
//!
//! **Nested & Composed Usage:**
//! Transformers can be evaluated inside arguments of other transformers. Inner expressions are always evaluated first.
//! * `[< a b c >]:intersperse[[[< x y >]:concatenate]]` -> `a xy b xy c`
//! * `[< a b >]:push_left[[[< hello world >]:concatenate]]` -> `helloworld a b`
//! * `[< greet >]:push_right[[[< hello world >]:case[[pascal]]]]` -> `greet HelloWorld`
//!
//! **Literal Transformations:**
//! Case transformations apply seamlessly to string literals and identifiers alike:
//! * `[< "hello" world >]:case[[snake]]` -> `hello world`
//!
//! # Remarks
//!
//! * [`Concatenate`] directly glues the textual representations of tokens together. Token groups are processed recursively, meaning any nested tokens are flattened into the final result.
//! * [`Case`] targets identifier-like tokens, string literals, and boolean literals. It safely preserves punctuation and non-identifier tokens where possible.

use std::{
    iter::{self, Peekable},
    str::FromStr,
};

use proc_macro2::{Group, Ident, Literal, TokenStream, TokenTree};

use quote::ToTokens;

use syn::{
    Lit,
    parse::{Nothing, Parse, ParseStream},
    spanned::Spanned,
};

use heck::{AsLowerCamelCase, AsPascalCase, AsSnekCase};

use tokel_engine::prelude::{Pass, Registry, Transformer};

/// A transformer that concatenates all elegible input tokens into a single identifier.
///
/// It ignores standard spacing and simply glues the string representations
/// of the tokens together.
///
/// By "token", this implies identifier and string literals (not including byte literals, c-strings, or other string type).
///
/// This performs a rolling approach, physically contiguous tokens of the same type will be concatenated into one of the same token type.
///
/// # Example
///
/// `[< hello _ world "what" "ever" . "buddy" >]:concatenate` -> `hello_world "whatever" . "buddy"`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Concatenate;

impl Pass for Concatenate {
    type Argument = Nothing;

    fn through(&mut self, input: TokenStream, _: Self::Argument) -> syn::Result<TokenStream> {
        // NOTE(invariant): None.
        struct ConcatIter(Peekable<<TokenStream as IntoIterator>::IntoIter>);

        impl ConcatIter {
            fn stream(stream: TokenStream) -> syn::Result<TokenStream> {
                let mut nested_iter = Self(stream.into_iter().peekable());

                let mut nested_tokens = Vec::new();

                loop {
                    match nested_iter.next() {
                        Some(Ok(tree)) => nested_tokens.push(tree),
                        Some(Err(error)) => return Err(error),
                        None => break,
                    }
                }

                Ok(nested_tokens.into_iter().collect::<TokenStream>())
            }
        }

        impl Iterator for ConcatIter {
            type Item = syn::Result<TokenTree>;

            fn next(&mut self) -> Option<Self::Item> {
                let Self(inner_iter) = self;

                match inner_iter.peek() {
                    Some(TokenTree::Ident(..) | TokenTree::Literal(..) | TokenTree::Group(..)) => {
                        match inner_iter.next() {
                            Some(TokenTree::Ident(ident_start)) => {
                                let mut ident_str = String::new();

                                let mut ident_tokens = TokenStream::new();

                                ident_str.push_str(ident_start.to_string().as_str());
                                ident_tokens
                                    .extend(iter::once(ident_start).map(Ident::into_token_stream));

                                while let Some(TokenTree::Ident(..)) = inner_iter.peek() {
                                    let Some(TokenTree::Ident(ident_extra)) = inner_iter.next()
                                    else {
                                        unreachable!()
                                    };

                                    ident_tokens.extend(
                                        iter::once(ident_extra.clone())
                                            .map(Ident::into_token_stream),
                                    );

                                    ident_str.push_str(ident_extra.to_string().as_str());
                                }

                                let mut ident = syn::parse_str::<Ident>(&ident_str).ok()?;

                                ident.set_span(ident_tokens.span());

                                Some(Ok(TokenTree::Ident(ident)))
                            }
                            Some(TokenTree::Literal(lit)) => {
                                if let Lit::Str(lit_str) = Lit::new(lit.clone()) {
                                    let mut concatenated_str = lit_str.value();

                                    while let Some(TokenTree::Literal(peeked_lit)) =
                                        inner_iter.peek()
                                    {
                                        if let Lit::Str(peeked_str) = Lit::new(peeked_lit.clone()) {
                                            let Some(..) = inner_iter.next() else {
                                                unreachable!()
                                            };

                                            concatenated_str.push_str(peeked_str.value().as_str());
                                        } else {
                                            break;
                                        }
                                    }

                                    let mut lit = Literal::string(&concatenated_str);

                                    lit.set_span(lit_str.span());

                                    Some(Ok(TokenTree::Literal(lit)))
                                } else {
                                    Some(Ok(TokenTree::Literal(lit)))
                                }
                            }
                            Some(TokenTree::Group(inner_group)) => {
                                let (delimiter, stream, span) = (
                                    inner_group.delimiter(),
                                    inner_group.stream(),
                                    inner_group.span(),
                                );

                                let stream = match Self::stream(stream) {
                                    Ok(stream) => stream,
                                    Err(error) => return Some(Err(error)),
                                };

                                let mut group = Group::new(delimiter, stream);

                                group.set_span(span);

                                Some(Ok(TokenTree::Group(group)))
                            }
                            Some(..) | None => unreachable!(),
                        }
                    }
                    Some(..) | None => inner_iter.next().map(Ok),
                }
            }
        }

        ConcatIter::stream(input)
    }
}

/// The target case style to transform the identifiers to.
#[derive(Debug, Copy, Clone)]
pub enum CaseStyle {
    /// `PascalCase`.
    Pascal,

    /// `camelCase`.
    Camel,

    /// `snake_case`.
    Snake,

    /// `UPPERCASE`
    Upper,

    /// `lowercase`
    Lower,
}

impl Parse for CaseStyle {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let case_ident = input.parse::<Ident>()?;

        let _: Nothing = input.parse()?;

        match case_ident.to_string().as_str() {
            "pascal" => Ok(Self::Pascal),
            "camel" => Ok(Self::Camel),
            "snake" => Ok(Self::Snake),
            "upper" => Ok(Self::Upper),
            "lower" => Ok(Self::Lower),
            _ => Err(syn::Error::new_spanned(
                case_ident,
                "unsupported case, supported ones are: `pascal`, `camel`, `snake`, `upper`, `lower`",
            )),
        }
    }
}

/// A transformer that changes the case of incoming identifiers, as instructed.
///
/// # Example
///
/// `[< hello _ world >]:case[[pascal]]` -> `Hello _ World`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Case;

impl Pass for Case {
    type Argument = CaseStyle;

    fn through(&mut self, input: TokenStream, style: Self::Argument) -> syn::Result<TokenStream> {
        fn apply_case(string: String, case: CaseStyle) -> String {
            match case {
                CaseStyle::Pascal => AsPascalCase(string).to_string(),
                CaseStyle::Camel => AsLowerCamelCase(string).to_string(),
                CaseStyle::Snake => AsSnekCase(string).to_string(),
                CaseStyle::Upper => string.to_uppercase(),
                CaseStyle::Lower => string.to_lowercase(),
            }
        }

        fn apply(input: TokenStream, case: CaseStyle) -> syn::Result<TokenStream> {
            input
                .into_iter()
                .try_fold(TokenStream::new(), |mut acc, target_tree| {
                    let target_output = match target_tree {
                        TokenTree::Literal(target_lit) => {
                            match syn::parse2::<Lit>(target_lit.into_token_stream())? {
                                Lit::Str(inner) => {
                                    TokenStream::from_str(apply_case(inner.value(), case).as_str())?
                                }
                                Lit::Bool(lit) => TokenStream::from_str(
                                    apply_case(lit.value.to_string(), case).as_str(),
                                )?,

                                lit => lit.into_token_stream(),
                            }
                        }
                        TokenTree::Ident(target_ident) => TokenStream::from_str(
                            apply_case(target_ident.to_string(), case).as_str(),
                        )?,
                        TokenTree::Group(group) => group
                            .stream()
                            .into_iter()
                            .map(|tree| apply(tree.into_token_stream(), case))
                            .try_fold(TokenStream::new(), |mut acc, result| {
                                result.map(|stream| {
                                    acc.extend(stream);
                                    acc
                                })
                            })
                            .map(|a| {
                                let mut new_group = Group::new(group.delimiter(), a);

                                new_group.set_span(group.span());

                                new_group
                            })
                            .map(TokenTree::Group)
                            .map(ToTokens::into_token_stream)?,

                        target_tree @ TokenTree::Punct(_) => target_tree.into_token_stream(),
                    };

                    acc.extend(target_output);

                    Ok(acc)
                })
        }

        apply(input, style)
    }
}

/// A transformer that converts every non-nested token tree into a string.
///
/// This does not further modify literals that are already strings.
///
/// # Example
///
/// `[< hello _ world >]:to_string` -> `"hello" "_" "world"`
pub struct ToString;

impl Pass for ToString {
    type Argument = syn::parse::Nothing;

    fn through(&mut self, input: TokenStream, _: Self::Argument) -> syn::Result<TokenStream> {
        // NOTE(invariant): None.
        struct ToStringIter(<TokenStream as IntoIterator>::IntoIter);

        impl ToStringIter {
            fn stream(stream: TokenStream) -> TokenStream {
                Self(stream.into_iter()).collect::<TokenStream>()
            }
        }

        impl Iterator for ToStringIter {
            type Item = TokenTree;

            fn next(&mut self) -> Option<Self::Item> {
                let Self(inner_iter) = self;

                let token_tree = inner_iter.next()?;

                Some(match token_tree {
                    TokenTree::Group(group) => {
                        let (delimiter, stream, span) =
                            (group.delimiter(), group.stream(), group.span());

                        let mut group = Group::new(delimiter, Self::stream(stream));

                        group.set_span(span);

                        TokenTree::Group(group)
                    }
                    TokenTree::Ident(ident) => {
                        let mut lit = Literal::string(ident.to_string().as_str());

                        lit.set_span(ident.span());

                        TokenTree::Literal(lit)
                    }
                    TokenTree::Punct(punct) => {
                        let mut lit = Literal::string(punct.to_string().as_str());

                        lit.set_span(punct.span());

                        TokenTree::Literal(lit)
                    }
                    TokenTree::Literal(literal) => {
                        // NOTE: If already a string-like literal, keep it as it is.
                        if let Lit::CStr(..) | Lit::ByteStr(..) | Lit::Char(..) | Lit::Str(..) =
                            Lit::new(literal.clone())
                        {
                            TokenTree::Literal(literal)
                        } else {
                            let mut lit = Literal::string(literal.to_string().as_str());

                            lit.set_span(literal.span());

                            TokenTree::Literal(lit)
                        }
                    }
                })
            }
        }

        Ok(ToStringIter::stream(input))
    }
}

/// A transformer that parses string literals back into token streams.
///
/// Non-string tokens are preserved. Groups are processed recursively.
///
/// # Errors
///
/// Returns an error when a string literal does not contain valid Rust tokens.
///
/// # Example
///
/// `[< "hello_world" >]:unstringify` -> `hello_world`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unstringify;

impl Pass for Unstringify {
    type Argument = Nothing;

    fn through(&mut self, input: TokenStream, _: Self::Argument) -> syn::Result<TokenStream> {
        fn apply(input: TokenStream) -> syn::Result<TokenStream> {
            input
                .into_iter()
                .try_fold(TokenStream::new(), |mut output, tree| {
                    let transformed = match tree {
                        TokenTree::Group(group) => {
                            let (delimiter, stream, span) =
                                (group.delimiter(), group.stream(), group.span());

                            let mut group = Group::new(delimiter, apply(stream)?);

                            group.set_span(span);
                            group.into_token_stream()
                        }
                        TokenTree::Literal(literal) => match Lit::new(literal.clone()) {
                            Lit::Str(string) => TokenStream::from_str(string.value().as_str())
                                .map_err(|error| {
                                    syn::Error::new(string.span(), error.to_string())
                                })?,
                            _ => literal.into_token_stream(),
                        },
                        tree => tree.into_token_stream(),
                    };

                    output.extend(transformed);

                    Ok(output)
                })
        }

        apply(input)
    }
}

/// Inserts all `string`-related [`Transformer`]s into the specified [`Registry`].
///
/// # Errors
///
/// This will fail if at least one standard `string`-related [`Transformer`] is already present by-name in the [`Registry`].
///
/// On failure, there is no guarantee that other non-colliding transformers have not been registered.
#[inline]
pub fn register(registry: &mut Registry) -> Result<(), Box<dyn Transformer>> {
    registry
        .try_insert("concatenate", Concatenate)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("case", Case)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("to_string", ToString)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    registry
        .try_insert("unstringify", Unstringify)
        .map_err(Box::new)
        .map_err(|t| t as Box<dyn Transformer>)?;

    Ok(())
}
