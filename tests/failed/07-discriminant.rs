use structified_enum::structify;

#[structify]
enum Foo {
    A = 2,
    B = 0,
    C,
    D,
}

fn main() {}