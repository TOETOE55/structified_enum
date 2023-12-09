use structified_enum::structify;

#[structify]
#[derive(Debug)]
enum Foo { A }

#[test]
fn test_debug_impl() {
    let a = format!("{:?}", Foo::A);
    let a1 = format!("{:?}", Foo::new(0));
    let unknown = format!("{:?}", Foo::new(1));

    assert_eq!(a, "A");
    assert_eq!(a1, "A");
    assert_eq!(unknown, "Foo(1)");
}

fn main() {}