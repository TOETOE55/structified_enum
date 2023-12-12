use structified_enum::structify;

#[structify]
#[derive(Debug)]
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
    assert!(matches!(Foo::try_from("A".to_string()), Ok(Foo::A)));
    assert!(matches!(Foo::try_from("B".to_string()), Ok(Foo::B)));
    assert!(Foo::try_from("C".to_string()).is_err());
    let aa: String = Foo::A.try_into().unwrap();
    assert_eq!(aa, "A".to_string());
}

#[test]
fn test_eq() {
    assert_eq!(Foo::A, Foo::new(0));
    assert_eq!(Foo::B, Foo::new(1));
}

fn main() {}
