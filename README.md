# `structified_enum`

This crate provides an attribute macro `structify` transforming a unit-like enum to a struct with its discriminant.

The following code:

```rust
use structified_enum::structify;

#[structify]
#[repr(u8)]
#[derive(Copy, Clone)]
enum Foo {
    A = 0,
    B,
    C,
}
```

is equivalent to

```rust
// #[repr(ty)] -> #[repr(transparent)]
#[repr(transparent)] 
#[derive(Copy, Clone)]
struct Foo(u8);

impl Foo {
    pub const A: Self = Self(0);
    pub const B: Self = Self(1);
    pub const C: Self = Self(2);
    
    pub fn new(value: u8) -> Self {
        Self(value)
    }
    
    // like `Foo::A as u8`
    pub fn value(self) -> u8 {
        self.0
    }
}
```

## Motivation

There are two main reasons:

1. Enum cannot be directly converted from its discriminant to its value. 
   It must be converted through `unsafe` or crates like [num-derive](https://crates.io/crates/num-derive) to generate conversion methods. 
   Because structures like `struct Foo(repr_ty)` can naturally express this conversion.
2. The backwards compatibility of enum is not good. 
   When a new version value is passed into an old version of the program (for instance, via serialization and deserialization), 
   the newly added variant cannot be recognized in the old version, its value will be discarded, making recovery impossible. 
   By explicitly recording its discriminant, information can be preserved for recovery when returned to the new version.

