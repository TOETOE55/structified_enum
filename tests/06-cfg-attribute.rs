use structified_enum::structify;

#[cfg(any(test, not(test)))]
#[structify]
#[cfg(any(test, not(test)))]
#[derive(PartialEq, Eq, Debug)]
#[cfg(any(test, not(test)))]
enum Foo {
    #[cfg(any(test, not(test)))]
    A,
    B
}

fn main() {}