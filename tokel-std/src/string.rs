//! String and text-manipulation Tokel [`Transformer`]s.

use std::str::FromStr;

use heck::{AsLowerCamelCase, AsPascalCase, AsSnekCase};
use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};

use quote::ToTokens;
use syn::{
    Lit,
    parse::{Nothing, Parse, ParseStream},
};

use tokel_engine::prelude::{Registry, Transformer};

/// A transformer that concatenates all input tokens into a single identifier.
///
/// It ignores standard spacing and simply glues the string representations
/// of the tokens together.
///
/// # Example
///
/// `[< hello _ world >]:concatenate` -> `hello_world`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Concatenate;

impl Transformer for Concatenate {
    fn transform(
        &mut self,
        input: TokenStream,
        argument: TokenStream,
    ) -> Result<TokenStream, syn::Error> {
        // Concatenate takes no arguments, so we enforce that the `[[...]]` is empty.
        let _: Nothing = syn::parse2(argument)?;

        // If the input is completely empty, just return empty.
        if input.is_empty() {
            return Ok(input);
        }

        let mut concatenated_string = String::new();
        let mut first_span = Span::call_site();
        let mut is_first = true;

        for tree in input {
            if is_first {
                first_span = tree.span();

                is_first = false;
            }

            // `to_string()` on a TokenTree strips `r#` from idents and handles raw strings nicely.
            concatenated_string.push_str(&tree.to_string());
        }

        // We must ensure the resulting string is a valid Rust identifier.
        // `syn::Ident::new` will panic if the string is not a valid ident (e.g. if it starts with a number).
        // To be safe, we try to parse it. If it fails, we return a syn::Error.
        let parsed_ident = syn::parse_str::<Ident>(&concatenated_string).map_err(|_| {
            syn::Error::new(
                first_span,
                format!("concatenated string `{concatenated_string}` is not a valid identifier"),
            )
        })?;

        Ok(quote::quote_spanned!(first_span=> #parsed_ident))
    }
}

/// A transformer that changes the case of incoming identifiers, as instructed.
///
/// # Example
///
/// `[< hello _ world >]:case[[pascal]]` -> `Hello _ World`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Case;

impl Transformer for Case {
    fn transform(&mut self, input: TokenStream, argument: TokenStream) -> syn::Result<TokenStream> {
        #[derive(Debug, Copy, Clone)]
        enum Target {
            Pascal,
            Camel,
            Snake,
        }

        impl Parse for Target {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let case_ident = input.parse::<Ident>()?;

                let _: Nothing = input.parse()?;

                Ok(match case_ident.to_string().as_str() {
                    "pascal" => Self::Pascal,
                    "camel" => Self::Camel,
                    "snake" => Self::Snake,
                    _ => return Err(syn::Error::new_spanned(case_ident, "unsupported case")),
                })
            }
        }

        let target_case: Target = syn::parse2(argument)?;

        fn apply_case(string: String, case: Target) -> String {
            match case {
                Target::Pascal => AsPascalCase(string).to_string(),
                Target::Camel => AsLowerCamelCase(string).to_string(),
                Target::Snake => AsSnekCase(string).to_string(),
            }
        }

        fn apply(input: TokenStream, case: Target) -> syn::Result<TokenStream> {
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

                                lit @ _ => lit.into_token_stream(),
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

                        target_tree @ _ => target_tree.into_token_stream(),
                    };

                    acc.extend(target_output);

                    Ok(acc)
                })
        }

        apply(input, target_case)
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

    Ok(())
}
