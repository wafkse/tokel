//! The Abstract Syntax Tree (AST) for Tokel.
//!
//! This module contains the data structures and `syn::parse::Parse` implementations
//! that represent the Tokel grammar. It is responsible for taking a raw
//! `proc_macro2::TokenStream` and structuring it into a recursive AST that can be
//! evaluated bottom-up by the engine.
//!
//! # Grammar Mapping
//!
//! The types in this module directly correspond to the formal EBNF grammar:
//!
//! * **`TokelStream`**: The root sequence of elements. Represents `TokelStream ::= Element*`.
//! * **`Element`**: An individual unit in the stream. It is either a standard Rust token tree
//!   (`TokelTree`) or an expansion block (`[ Block ] Pipeline`).
//! * **`Block`**: The inner contents of an expansion block (the `< TokelStream >` part).
//! * **`Pipeline`**: A sequence of one or more transformers attached to the end of an
//!   expansion block (e.g., `:case[[pascal]]:prefix[[Get]]`).
//! * **`Transformer`**: A single transformation operation (like `case`) and its optional
//!   arguments parsed from double brackets (`[[ ... ]]`).
//!
//! # Parsing Strategy
//!
//! The parser performs a deep traversal of the incoming token stream. When it encounters
//! standard Rust delimiters (`()`, `{}`, or `[]` that do not start with `<`), it recursively
//! parses their inner contents as a new `TokelStream`.
//!
//! This recursive parsing ensures that expansion blocks deeply nested inside standard Rust
//! code, `None`-delimited macro capture groups, or even inside transformer arguments, are
//! accurately located and represented in the final AST.

use std::{iter, ops::Not};

use syn::{
    Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
    token::Bracket,
};

use proc_macro2::{Delimiter, Ident, Literal, Punct, Span, TokenStream, TokenTree};

/// The `:`-punctuated sequence of [`Pipe`], to form a complete pipeline for an expansion block.
#[derive(Debug, Clone)]
pub struct Pipeline(pub Vec<Pipe>);

impl Parse for Pipeline {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut target_list = Vec::new();

        while input.peek(Token![:]) {
            target_list.push(input.parse::<Pipe>()?);
        }

        Ok(Self(target_list))
    }
}

/// A pass of a single transformer with the provided argument.
#[derive(Debug, Clone)]
pub struct Pipe {
    /// The preceding `:` token.
    pub colon_token: Token![:],

    /// The name of the transformer that is to be used within the active context.
    pub name: Ident,

    /// The argument that has been passed to the respective transformer.
    pub argument: Option<((Bracket, Bracket), TokelStream)>,
}

impl Parse for Pipe {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let colon_token: Token![:] = input.parse()?;

        let name: Ident = input.parse()?;

        let argument = None;

        // NOTE: Determine whether we are looking at a `[[...]]` textually.
        if input.peek(Bracket) {
            let fork = input.fork();

            let content_left;

            let bracket_left = syn::bracketed!(content_left in fork);

            if content_left.peek(Bracket) {
                // NOTE: Advance to the fork, as it was indeed an argument.
                input.advance_to(&fork);

                let content_right;

                let bracket_right = syn::bracketed!(content_right in content_left);

                let argument = (
                    (bracket_left, bracket_right),
                    content_right.parse::<TokelStream>()?,
                );

                let argument = Some(argument);

                Ok(Self {
                    colon_token,
                    name,
                    argument,
                })
            } else
            /* NOTE: Inner token-stream does not contain another bracket, ignore the top-level bracketed group. */
            {
                Ok(Self {
                    colon_token,
                    name,
                    argument,
                })
            }
        } else {
            Ok(Self {
                colon_token,
                name,
                argument,
            })
        }
    }
}

/// An expansion block.
#[derive(Debug, Clone)]
pub struct Block {
    /// The less-than token part of this expansion block.
    pub lt_token: Token![<],

    /// The tokel-specific token-stream contained within this expansion block.
    pub stream: TokelStream,

    /// The greater-than token part of this expansion block.
    pub gt_token: Token![>],
}

impl Parse for Block {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lt_token = input.parse()?;

        let mut tree_list = Vec::new();

        while input.is_empty().not() {
            tree_list.push(input.parse::<TokenTree>()?);
        }

        let Some(last_tree) = tree_list.pop() else {
            return Err(syn::Error::new_spanned(
                lt_token,
                "missmatched chevron token",
            ));
        };

        let gt_token = {
            let mut token_stream = TokenStream::new();

            token_stream.extend(iter::once(last_tree));

            syn::parse2(token_stream)?
        };

        let stream = {
            let mut token_stream = TokenStream::new();

            token_stream.extend(tree_list.into_iter());

            syn::parse2(token_stream)?
        };

        Ok(Self {
            lt_token,
            stream,
            gt_token,
        })
    }
}

/// An element inside a tokel-processed token-stream.
///
/// An element can be either an expansion block or a token tree.
#[derive(Debug, Clone)]
pub enum Element {
    /// An expansion block in a tokel-stream.
    Block {
        /// The expansion block content.
        block: Block,

        /// The pipeline to be used for the token-stream.
        pipeline: Option<Pipeline>,
    },

    /// An unmodified tokel-tree.
    Tree(TokelTree),
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<TokenTree>()? {
            TokenTree::Group(group) => {
                let mut iter = group.stream().into_iter();

                iter.next()
                    .map(|first| iter.last().map(|last| (first, last)))
                    .flatten()
                    .and_then(|(first, last)| match (first, last) {
                        (TokenTree::Punct(left), TokenTree::Punct(right))
                            if left.as_char() == '<' && right.as_char() == '>' =>
                        {
                            Some(syn::parse2::<Block>(group.stream()).map(|block| {
                                (input.peek(Token![:]) && input.peek2(syn::Ident))
                                    .then(|| input.parse::<Pipeline>())
                                    .transpose()
                                    .map(|pipeline| Element::Block { block, pipeline })
                            }))
                        }
                        _ => None,
                    })
                    .filter(|_| group.delimiter() == Delimiter::Bracket)
                    .transpose()?
                    .unwrap_or_else(|| {
                        let (delimiter, span, tokens) =
                            (group.delimiter(), group.span(), group.stream());

                        let stream = syn::parse2(tokens)?;

                        Ok(Self::Tree(TokelTree::Group(TokelGroup {
                            delimiter,
                            span,
                            stream,
                        })))
                    })
            }
            target_tree => {
                let target_variant = match target_tree {
                    TokenTree::Ident(ident) => TokelTree::Ident(ident),
                    TokenTree::Punct(punct) => TokelTree::Punct(punct),
                    TokenTree::Literal(literal) => TokelTree::Literal(literal),
                    TokenTree::Group(..) => unreachable!(),
                };

                Ok(Self::Tree(target_variant))
            }
        }
    }
}

/// A tokel-specific `TokenTree`.
///
/// This is used to perform recursive-descent expansion of all tokens.
#[derive(Debug, Clone)]
pub enum TokelTree {
    /// A delimited sequence of tokens.
    Group(TokelGroup),

    /// An identifier.
    Ident(Ident),

    /// A literal.
    Literal(Literal),

    /// A punctuation fragment.
    Punct(Punct),
}

/// A tokel-specific `TokenGroup`.
#[derive(Debug, Clone)]
pub struct TokelGroup {
    /// The delimiter of this tokel-group.
    ///
    /// Note that any `None`-delimited [`TokenTree`] is acknowledged and will be processed verbatim.
    pub delimiter: Delimiter,

    /// A delimiter span.
    pub span: Span,

    /// The tokel-specific token-stream that this tokel-group embeds.
    pub stream: TokelStream,
}

/// A tokel-specific stream of source-level elements.
///
/// See the [`Element`] item for further information.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct TokelStream(pub Vec<Element>);

impl Parse for TokelStream {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut target_list = Vec::new();

        loop {
            if input.is_empty() {
                break Ok(Self(target_list));
            }

            target_list.push(input.parse()?);
        }
    }
}
