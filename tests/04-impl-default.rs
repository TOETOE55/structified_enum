use structified_enum::structify;

#[structify]
#[derive(Default)]
enum Empty {}

#[structify]
#[derive(Default)]
enum Zero {
    A,
}

#[structify]
#[derive(Default)]
enum Neg1 {
    A = -1,
}

#[test]
fn test_impl_default() {
    assert_eq!(Empty::default().value(), 0);
    assert_eq!(Zero::default().value(), 0);
    assert_eq!(Neg1::default().value(), -1);
}

fn main() {}
