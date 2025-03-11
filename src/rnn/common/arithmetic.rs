use std::{
    fmt::Debug,
    iter::Sum,
    ops::{Add, AddAssign, Mul, SubAssign},
};

pub trait Arithmetic:
    Default
    + Debug
    + Copy
    + Send
    + Sync
    + Ord
    + Add<Output = Self>
    + Mul<Output = Self>
    + Sum
    + SubAssign
    + AddAssign
    + 'static
{
    fn one() -> Self;
}

impl Arithmetic for u8 {
    fn one() -> Self {
        1_u8
    }
}

impl Arithmetic for i8 {
    fn one() -> Self {
        1_i8
    }
}

impl Arithmetic for i16 {
    fn one() -> Self {
        1_i16
    }
}

impl Arithmetic for u16 {
    fn one() -> Self {
        1_u16
    }
}

impl Arithmetic for i32 {
    fn one() -> Self {
        1_i32
    }
}

impl Arithmetic for u32 {
    fn one() -> Self {
        1_u32
    }
}

impl Arithmetic for i64 {
    fn one() -> Self {
        1_i64
    }
}

impl Arithmetic for u64 {
    fn one() -> Self {
        1_u64
    }
}

impl Arithmetic for usize {
    fn one() -> Self {
        1_usize
    }
}

impl Arithmetic for isize {
    fn one() -> Self {
        1_isize
    }
}
