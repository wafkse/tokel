use std::borrow::Cow;
use std::iter;

use proc_macro2::{Group, TokenStream, TokenTree};
use syn::Ident;

use crate::session::Session;

use crate::syntax::{Block, Element, Pipe, Pipeline, TokelGroup, TokelStream, TokelTree};

/// A trait for evaluating Tokel AST nodes into standard token streams.
///
/// This trait allows any individual part of the Tokel grammar (from a full `TokelStream`
/// down to a single `Pipeline`) to be evaluated dynamically using the state held
/// within a [`Session`].
pub trait Expand {
    /// Additional context or data required to expand this specific node.
    ///
    /// For self-contained nodes like `TokelStream` or `Element`, this is simply `()`.
    /// For modifier nodes like `Pipeline`, this is the `TokenStream` being transformed.
    type Context;

    /// Expands the AST node into a fully resolved token stream using explicit context.
    ///
    /// This is the core evaluation method. It traverses the node, looking up and
    /// applying any transformers from the provided [`Session`], and utilizing the
    /// provided `context` to complete its evaluation.
    fn expand_with(
        self,
        session: &mut Session,
        context: Self::Context,
    ) -> Result<TokenStream, syn::Error>;

    /// Expands the AST node into a fully resolved token stream.
    ///
    /// This is a convenience method available for nodes that do not require complex
    /// external context (i.e., where `Context` implements `Default`, such as `()`).
    /// It automatically calls [`expand_with`](Self::expand_with) using the default context.
    fn expand(self, session: &mut Session) -> Result<TokenStream, syn::Error>
    where
        Self::Context: Default,
        Self: Sized,
    {
        Self::expand_with(self, session, Self::Context::default())
    }
}

impl Expand for TokelStream {
    type Context = ();

    fn expand_with(
        self,
        session: &mut Session,
        _: Self::Context,
    ) -> Result<TokenStream, syn::Error> {
        let Self(input_vector) = self;

        let mut output_stream = TokenStream::new();

        for input_element in input_vector {
            output_stream.extend(input_element.expand(session)?);
        }

        Ok(output_stream)
    }
}

impl Expand for Element {
    type Context = ();

    fn expand_with(
        self,
        session: &mut Session,
        _: Self::Context,
    ) -> Result<TokenStream, syn::Error> {
        match self {
            Element::Block {
                block: Block { stream, .. },
                pipeline: Some(pipeline),
            } => pipeline.expand_with(session, stream),
            Element::Block {
                pipeline: None,
                block: Block { stream, .. },
            } => stream.expand(session),
            Element::Tree(tokel_tree) => tokel_tree.expand(session),
        }
    }
}

impl Expand for TokelTree {
    type Context = ();

    fn expand_with(
        self,
        session: &mut Session,
        _: Self::Context,
    ) -> Result<TokenStream, syn::Error> {
        match self {
            TokelTree::Group(tokel_group) => tokel_group.expand(session),
            tree => {
                let mut stream = TokenStream::new();

                stream.extend(iter::once(match tree {
                    TokelTree::Ident(ident) => TokenTree::Ident(ident),
                    TokelTree::Literal(literal) => TokenTree::Literal(literal),
                    TokelTree::Punct(punct) => TokenTree::Punct(punct),
                    TokelTree::Group(..) => unreachable!(),
                }));

                Ok(stream)
            }
        }
    }
}

impl Expand for TokelGroup {
    type Context = ();

    fn expand_with(
        self,
        session: &mut Session,
        _: Self::Context,
    ) -> Result<TokenStream, syn::Error> {
        let Self {
            delimiter,
            span,
            stream,
        } = self;

        let mut output_stream = TokenStream::new();

        let mut group = Group::new(delimiter, stream.expand(session)?);

        group.set_span(span);

        output_stream.extend(iter::once(TokenTree::Group(group)));

        Ok(output_stream)
    }
}

impl Expand for Pipeline {
    type Context = TokelStream;

    fn expand_with(
        self,
        session: &mut Session,
        body: Self::Context,
    ) -> Result<TokenStream, syn::Error> {
        let mut input = body.expand(session)?;

        let Self(pipe_list) = self;

        for Pipe {
            ref name, argument, ..
        } in pipe_list
        {
            let ref target_name = Cow::Owned(Ident::to_string(name));

            let argument = if let Some((.., argument)) = argument {
                argument.expand(session)
            } else {
                Ok(TokenStream::new())
            };

            if let Some(transformer) = session.registry_mut().get_mut(target_name) {
                input = transformer.transform(input, argument?)?;
            } else {
                return Err(syn::Error::new(
                    name.span(),
                    format!("no such transformer `{target_name}`"),
                ));
            }
        }

        Ok(input)
    }
}
