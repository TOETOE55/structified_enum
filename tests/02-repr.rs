use structified_enum::structify;

#[structify]
enum ImplicitRepr {
    A,
}

const _: i32 = ImplicitRepr::A.value();

#[structify]
#[repr(i8)]
enum I8Repr {
    A,
}

const _: i8 = I8Repr::A.value();

#[structify]
#[repr(u8)]
enum U8Repr {
    A,
}

const _: u8 = U8Repr::A.value();


#[structify]
#[repr(i16)]
enum I16Repr {
    A,
}

const _: i16 = I16Repr::A.value();

#[structify]
#[repr(u16)]
enum U16Repr {
    A,
}

const _: u16 = U16Repr::A.value();

#[structify]
#[repr(i32)]
enum I32Repr {
    A,
}

const _: i32 = I32Repr::A.value();

#[structify]
#[repr(u32)]
enum U32Repr {
    A,
}

const _: u32 = U32Repr::A.value();

#[structify]
#[repr(i64)]
enum I64Repr {
    A,
}

const _: i64 = I64Repr::A.value();

#[structify]
#[repr(u64)]
enum U64Repr {
    A,
}

const _: u64 = U64Repr::A.value();

#[structify]
#[repr(i128)]
enum I128Repr {
    A,
}

const _: i128 = I128Repr::A.value();

#[structify]
#[repr(u128)]
enum U128Repr {
    A,
}

const _: u128 = U128Repr::A.value();

#[structify]
#[repr(isize)]
enum IsizeRepr {
    A,
}

const _: isize = IsizeRepr::A.value();

#[structify]
#[repr(usize)]
enum UsizeRepr {
    A,
}

const _: usize = UsizeRepr::A.value();

#[structify]
#[repr(C, i32)]
enum Repr { A }

fn main() {}