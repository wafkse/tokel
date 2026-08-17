#![cfg(feature = "derive")]

tokel::stream! {
    #[derive(Debug, PartialEq)]
    struct [< generated_type >]:case[[pascal]] {
        value: usize,
    }

    const FIRST: u32 = [< >]:enumerate;
    const SECOND: u32 = [< >]:enumerate;
}

#[tokel::attribute(derive([< debug >]:case[[pascal]]))]
struct AttributeTarget;

#[test]
fn stream_expands_through_the_public_facade() {
    let left = GeneratedType { value: 7 };
    let right = GeneratedType { value: 7 };

    assert_eq!(left, right);
    assert_eq!((FIRST, SECOND), (0, 1));
}

#[test]
fn attribute_expands_its_arguments() {
    assert_eq!(format!("{AttributeTarget:?}"), "AttributeTarget");
}
