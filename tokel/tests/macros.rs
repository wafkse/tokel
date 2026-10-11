#![cfg(feature = "derive")]

tokel::stream! {
    #[derive(Debug, PartialEq)]
    struct [< generated_type >]:case[[pascal]] {
        value: usize,
    }

    const FIRST: u32 = [< >]:enumerate;
    const SECOND: u32 = [< >]:enumerate;

    const PACKAGE: &str = [< >]:environment[["CARGO_PKG_NAME"]];
    const MAJOR: u64 = [< >]:environment[["CARGO_PKG_VERSION_MAJOR"]]:unstringify;
    const NESTED_PACKAGE: &str = [< >]:environment[[[< "CARGO_" "PKG_NAME" >]:concatenate]];
}

macro_rules! captured_accessor {
    ($index:literal) => {
        tokel::stream! {
            pub fn [< captured_ $index >]:to_string:flatten:concatenate:unstringify() -> usize {
                $index
            }
        }
    };
}

captured_accessor!(31);

#[tokel::attribute(derive([< debug >]:case[[pascal]]))]
struct AttributeTarget;

#[test]
fn stream_expands_through_the_public_facade() {
    let left = GeneratedType { value: 7 };
    let right = GeneratedType { value: 7 };

    assert_eq!(left, right);
    assert_eq!((FIRST, SECOND), (0, 1));
    assert_eq!(captured_31(), 31);
}

#[test]
fn attribute_expands_its_arguments() {
    assert_eq!(format!("{AttributeTarget:?}"), "AttributeTarget");
}

#[test]
fn environment_expands_and_composes_through_the_public_facade() {
    assert_eq!(PACKAGE, env!("CARGO_PKG_NAME"));
    assert_eq!(NESTED_PACKAGE, PACKAGE);
    assert_eq!(MAJOR, 0);
}
