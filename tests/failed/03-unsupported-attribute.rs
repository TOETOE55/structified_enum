use structified_enum::structify;

#[structify]
#[other_attr]
enum Foo {}

#[structify]
enum Bar {
    #[other_attr]
    A
}

fn main() {}