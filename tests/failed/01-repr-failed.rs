use structified_enum::structify;

#[structify]
#[repr(i32, i64)]
enum DupTypeRepr { A }

#[structify]
#[repr(C, transparent)]
enum CTransRepr { A }

#[structify]
#[repr(transparent, transparent)]
enum DupTransRepr { A }

#[structify]
#[repr(u64)]
enum UnsizeRepr { A = -1 }

fn main() {}