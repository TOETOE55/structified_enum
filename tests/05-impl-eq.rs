use structified_enum::structify;

#[structify]
#[derive(PartialEq, Eq, Debug)]
enum Foo {
    A,
    B,
}

#[test]
fn test_pattern_match() {
    assert!(matches!(Foo::A, Foo::A));
    assert!(matches!(Foo::B, Foo::B));
    assert!(!matches!(Foo::A, Foo::B));
    assert!(!matches!(Foo::new(3), Foo::A));
}

#[test]
fn test_eq() {
    assert_eq!(Foo::A, Foo::new(0));
    assert_eq!(Foo::B, Foo::new(1));
}

fn main() {}
