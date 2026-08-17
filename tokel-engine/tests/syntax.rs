use proc_macro2::Delimiter;
use quote::quote;
use tokel_engine::syntax::*;

#[test]
fn parse_empty_stream() {
    let input = quote! {};
    let ast: TokelStream = syn::parse2(input).unwrap();
    assert!(ast.0.is_empty(), "Stream should have exactly 0 elements");
}

#[test]
fn parse_standard_rust_tokens() {
    let input = quote! { pub fn hello() {} };
    let ast: TokelStream = syn::parse2(input).unwrap();

    // `pub`, `fn`, `hello`, `()`, `{}`
    assert_eq!(ast.0.len(), 5);

    for element in ast.0 {
        match element {
            Element::Tree(_) => {}
            _ => panic!("Expected only standard trees, found an expansion block!"),
        }
    }
}

#[test]
fn parse_simple_chevron_block() {
    let input = quote! { [< a b c >] };
    let ast: TokelStream = syn::parse2(input).unwrap();

    assert_eq!(ast.0.len(), 1, "Should parse as exactly one element");

    match &ast.0[0] {
        Element::Block { block, pipeline } => {
            // Assert the inner block parsed 3 elements (a, b, c)
            assert_eq!(block.stream.0.len(), 3);
            // Assert there is no pipeline
            assert!(pipeline.is_none());
        }
        _ => panic!("Expected an Element::Block"),
    }
}

#[test]
fn parse_block_with_pipeline() {
    let input = quote! { [< x >]:case:append[[_suffix]] };
    let ast: TokelStream = syn::parse2(input).unwrap();

    match &ast.0[0] {
        Element::Block {
            block: _,
            pipeline: Some(pipeline),
        } => {
            assert_eq!(pipeline.0.len(), 2);

            // First transformer: `case`
            assert_eq!(pipeline.0[0].name.to_string(), "case");
            assert!(pipeline.0[0].argument.is_none());

            // Second transformer: `append`
            assert_eq!(pipeline.0[1].name.to_string(), "append");

            // Ensure arguments parsed correctly
            let args = pipeline.0[1]
                .argument
                .as_ref()
                .expect("Expected args for append");
            assert_eq!(args.1.0.len(), 1); // `_suffix`
        }
        _ => panic!("Expected an Element::Block with a Pipeline"),
    }
}

#[test]
fn parse_nested_blocks() {
    let input = quote! { [< [< inner >]:first >]:second };
    let ast: TokelStream = syn::parse2(input).unwrap();

    // Outer Block
    match &ast.0[0] {
        Element::Block {
            block: outer_block,
            pipeline: Some(outer_pipeline),
        } => {
            assert_eq!(outer_pipeline.0[0].name.to_string(), "second");

            // Inner Block
            assert_eq!(outer_block.stream.0.len(), 1);
            match &outer_block.stream.0[0] {
                Element::Block {
                    block: inner_block,
                    pipeline: Some(inner_pipeline),
                } => {
                    assert_eq!(inner_block.stream.0.len(), 1); // `inner`
                    assert_eq!(inner_pipeline.0[0].name.to_string(), "first");
                }
                _ => panic!("Expected a nested Element::Block"),
            }
        }
        _ => panic!("Expected an outer Element::Block"),
    }
}

#[test]
fn parse_recursive_group_traversal() {
    // We place a chevron block inside standard rust parenthesis `()`
    let input = quote! { ( [< target >]:transform ) };
    let ast: TokelStream = syn::parse2(input).unwrap();

    assert_eq!(ast.0.len(), 1);

    match &ast.0[0] {
        Element::Tree(TokelTree::Group(group)) => {
            assert_eq!(group.delimiter, Delimiter::Parenthesis);
            assert_eq!(group.stream.0.len(), 1);
            assert!(matches!(group.stream.0[0], Element::Block { .. }));
        }
        _ => panic!("Expected an Element::Tree(TokelTree::Group)"),
    }
}

#[test]
fn parse_block_with_mixed_contents() {
    // Tests that a block can hold standard rust tokens and inner blocks side-by-side
    let input = quote! { [< let x = [< inner >]; >] };
    let ast: TokelStream = syn::parse2(input).unwrap();

    match &ast.0[0] {
        Element::Block { block, .. } => {
            let inner_elements = &block.stream.0;
            assert_eq!(inner_elements.len(), 5);

            assert!(matches!(
                inner_elements[0],
                Element::Tree(TokelTree::Ident(_))
            )); // `let`
            assert!(matches!(
                inner_elements[1],
                Element::Tree(TokelTree::Ident(_))
            )); // `x`
            assert!(matches!(
                inner_elements[2],
                Element::Tree(TokelTree::Punct(_))
            )); // `=`
            assert!(matches!(inner_elements[3], Element::Block { .. })); // `[< inner >]`
            assert!(matches!(
                inner_elements[4],
                Element::Tree(TokelTree::Punct(_))
            )); // `;`
        }
        _ => panic!("Expected an Element::Block"),
    }
}

#[test]
fn parse_double_bracket_lookalike_ignore() {
    // Tests that standard standard multidimensional arrays don't get mistaken for pipelines
    let input = quote! { [[1, 2], [3, 4]] };
    let ast: TokelStream = syn::parse2(input).unwrap();

    // It should parse as a standard TokelTree::Group (outer bracket)
    // containing two TokelTree::Groups (inner brackets) separated by a comma.
    assert_eq!(ast.0.len(), 1);
    match &ast.0[0] {
        Element::Tree(TokelTree::Group(group)) => {
            assert_eq!(group.delimiter, Delimiter::Bracket);
            assert_eq!(group.stream.0.len(), 3); // `[1, 2]`, `,`, `[3, 4]`
        }
        _ => panic!("Expected a standard array grouping, not an expansion block"),
    }
}

#[test]
fn parse_looks_like_block_but_missing_gt() {
    // Here, we have `[<` but the bracketed group NEVER closes with `>`.
    // For example, this could be valid Rust code in an array: `[<MyType as Trait>::Assoc]`
    let input = quote! { [<MyType as Trait>::Assoc] };
    let ast: TokelStream = syn::parse2(input).unwrap();

    assert_eq!(ast.0.len(), 1);

    // Because the inner token stream of the bracket DOES NOT end in `>`,
    // it must fall back to a standard `TokelTree::Group`.
    match &ast.0[0] {
        Element::Tree(TokelTree::Group(group)) => {
            assert_eq!(group.delimiter, Delimiter::Bracket);
            // Inside the standard group, we should have `<`, `MyType`, `as`, etc.
            assert_eq!(group.stream.0.len(), 8);
            assert!(
                matches!(group.stream.0[0], Element::Tree(TokelTree::Punct(ref p)) if p.as_char() == '<')
            );
        }
        _ => panic!("Parser incorrectly interpreted a path-in-array as an expansion block!"),
    }
}

#[test]
fn parse_looks_like_block_but_missing_lt() {
    // Starts with a standard identifier, ends with `>`.
    // Example: `[ T::Assoc > ]` (Not valid Rust usually, but valid TokenStream).
    let input = quote! { [ T::Assoc > ] };
    let ast: TokelStream = syn::parse2(input).unwrap();

    assert_eq!(ast.0.len(), 1);

    match &ast.0[0] {
        Element::Tree(TokelTree::Group(group)) => {
            assert_eq!(group.delimiter, Delimiter::Bracket);
            // We ensure it didn't accidentally consume the `>` into the void.
            assert!(matches!(
                group.stream.0.last().unwrap(),
                Element::Tree(TokelTree::Punct(p)) if p.as_char() == '>'
            ));
        }
        _ => panic!(
            "Parser interpreted a sequence ending in `>` as an expansion block without a starting `<`"
        ),
    }
}

#[test]
fn parse_pipeline_lookalike_on_standard_group() {
    // We have a standard bracketed group, immediately followed by `:ident`.
    // Since the group does NOT start with `<` and end with `>`, it should NOT parse as a block.
    // The `:ident` should just be parsed as standard tokens.
    let input = quote! { [ standard_rust_array ]:transform };
    let ast: TokelStream = syn::parse2(input).unwrap();

    // Should parse as THREE separate elements: the Group `[]`, the Punct `:`, the Ident `transform`
    assert_eq!(ast.0.len(), 3);

    assert!(matches!(ast.0[0], Element::Tree(TokelTree::Group(_))));
    assert!(matches!(ast.0[1], Element::Tree(TokelTree::Punct(ref p)) if p.as_char() == ':'));
    assert!(matches!(ast.0[2], Element::Tree(TokelTree::Ident(_))));
}

#[test]
fn parse_valid_block_with_complex_inner_rust_code() {
    // An expansion block that contains valid Rust code that uses `<` and `>` internally.
    // The parser must NOT stop at the inner `>`, it must find the LAST `>`.
    let input = quote! { [< HashMap::<String, Vec<u8>>::new() >] };
    let ast: TokelStream = syn::parse2(input).unwrap();

    match &ast.0[0] {
        Element::Block { block, pipeline } => {
            assert!(pipeline.is_none());

            // The inner stream should be `HashMap`, `::`, `<`, `String`, `,`, `Vec`, `<`, `u8`, `>`, `>`, `::`, `new`, `()`.
            // If the parser stopped at the first `>`, this assertion will fail.
            let inner = &block.stream.0;

            // Verify the last tokens before the closing `>` of the block are `::`, `new`, `()`
            assert!(
                matches!(inner[inner.len() - 1], Element::Tree(TokelTree::Group(ref g)) if g.delimiter == Delimiter::Parenthesis)
            );
            assert!(
                matches!(inner[inner.len() - 2], Element::Tree(TokelTree::Ident(ref i)) if i == "new")
            );
        }
        _ => panic!("Failed to parse a block containing inner angle brackets"),
    }
}

#[test]
fn parse_nested_pipeline_arguments() {
    // The argument `[[ ... ]]` to a transformer is actually another pipeline!
    let input = quote! { [< main >]:append[[ [< suffix >]:case ]] };
    let ast: TokelStream = syn::parse2(input).unwrap();

    match &ast.0[0] {
        Element::Block {
            pipeline: Some(pipeline),
            ..
        } => {
            let pipe = &pipeline.0[0];
            let args_stream = &pipe.argument.as_ref().unwrap().1;

            // The argument stream should contain exactly one element: The inner `Element::Block`
            assert_eq!(args_stream.0.len(), 1);

            match &args_stream.0[0] {
                Element::Block {
                    block: inner_block,
                    pipeline: Some(inner_pipeline),
                } => {
                    assert_eq!(inner_pipeline.0[0].name.to_string(), "case");

                    // And the inner block contains `suffix`
                    match &inner_block.stream.0[0] {
                        Element::Tree(TokelTree::Ident(i)) => assert_eq!(i, "suffix"),
                        _ => panic!("Expected identifier 'suffix'"),
                    }
                }
                _ => panic!("Expected the argument to parse as an inner expansion block"),
            }
        }
        _ => panic!("Expected outer block"),
    }
}
