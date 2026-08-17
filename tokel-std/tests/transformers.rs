use proc_macro2::TokenStream;
use quote::quote;
use tokel_engine::prelude::{Expand, Session, TokelStream};

fn expand(input: TokenStream) -> TokenStream {
    let mut session = Session::new();

    assert!(tokel_std::register(session.registry_mut()).is_ok());

    let stream = syn::parse2::<TokelStream>(input).expect("test input should parse");

    stream
        .expand(&mut session)
        .expect("test input should expand")
}

fn assert_streams_eq(actual: TokenStream, expected: TokenStream) {
    assert_eq!(actual.to_string(), expected.to_string());
}

#[test]
fn iteration_transformers_compose() {
    let actual = expand(quote! {
        [< a b c >]:reverse:intersperse[[,]]
    });
    let expected = quote! { c, b, a };

    assert_streams_eq(actual, expected);
}

#[test]
fn string_transformers_compose() {
    let actual = expand(quote! {
        [< hello _ world >]:concatenate:case[[pascal]]
    });
    let expected = quote! { HelloWorld };

    assert_streams_eq(actual, expected);
}

#[test]
fn structural_transformers_compose() {
    let actual = expand(quote! {
        [< (a) [b] {c} >]:flatten:encapsulate[[()]]
    });
    let expected = quote! { (a b c) };

    assert_streams_eq(actual, expected);
}

#[test]
fn state_is_scoped_to_one_session() {
    let actual = expand(quote! {
        [< >]:enumerate [< >]:enumerate
    });
    let expected = quote! { 0 1 };

    assert_streams_eq(actual, expected);
}

#[test]
fn standard_registration_rejects_collisions() {
    let mut session = Session::new();

    assert!(tokel_std::register(session.registry_mut()).is_ok());

    assert!(tokel_std::register(session.registry_mut()).is_err());
}
