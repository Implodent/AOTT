use std::{mem::size_of, ptr::addr_of};

use aott::{
        bytes::number::*,
        input::Input,
        parser::{Parser, SimpleExtras},
        primitive::{filter, take, take_exact},
        IResult,
};
use zstd_safe::ErrorCode;

fn zstd(bytes: &[u8]) -> Result<Vec<u8>, ErrorCode> {
        let mut vec = vec![];
        zstd_safe::decompress(&mut vec, bytes).map(|_| vec)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum NbtTag {
        End = 0,
        Byte = 1,
        Short = 2,
        Int = 3,
        Long = 4,
        Float = 5,
        Double = 6,
        ByteArray = 7,
        Str = 8,
        List = 9,
        Compound = 10,
        IntArray = 11,
        LongArray = 12,
}

macro_rules! define_len {
    (|$n:ident| $($name:ident $real:tt),*) => {
        match $n {
            $(Self::$name => define_len!(~ $real),)*
        }
    };
    (~ null) => {
        None
    };
    (~ $t:ty) => {
        Some(std::mem::size_of::<$t>())
    };
    (~ $val:literal) => {
        Some($val)
    };
}

impl NbtTag {
        pub fn from_u8(n: u8) -> Option<Self> {
                (n < 12).then(|| unsafe { addr_of!(n).cast::<Self>().read() })
        }
        pub fn to_u8(&self) -> u8 {
                *self as u8
        }
        pub fn payload_len(&self) -> Option<usize> {
                define_len! { |self|
                        End 0,
                        Byte u8,
                        Short u16,
                        Int u32,
                        Long u64,
                        Float f32,
                        Double f64,
                        ByteArray null,
                        Str null,
                        List null,
                        Compound null,
                        IntArray null,
                        LongArray null
                }
        }
}

#[derive(Debug, Clone)]
struct RawNbt {
        tag: NbtTag,
        name: String,
        length: usize,
        payload: Vec<u8>,
}

fn raw_nbt<'a, 'input>(
        input: Input<'input, &'a [u8]>,
) -> IResult<'input, &'a [u8], SimpleExtras<&'a [u8]>, RawNbt> {
        let (input, tag) = filter(|n: &u8| *n < 12)
                .map(|n| NbtTag::from_u8(n).unwrap())
                .parse(input)?;

        if tag == NbtTag::End {
                return Ok((
                        input,
                        RawNbt {
                                tag,
                                name: String::new(),
                                length: 0,
                                payload: vec![],
                        },
                ));
        }

        let (input, name_len) = big::u16.map(|u| u as usize).parse(input)?;
        let (input, name) = take(name_len)
                .map(|utf8| unsafe { String::from_utf8_unchecked(utf8) })
                .parse(input)?;

        if let Some(fixed_length) = tag.payload_len() {
                let (input, payload) = take(fixed_length).parse(input)?;

                Ok((
                        input,
                        RawNbt {
                                tag,
                                name,
                                length: fixed_length,
                                payload,
                        },
                ))
        } else {
                use NbtTag::*;
                let (input, (length, payload)) = match tag {
                        End | Byte | Short | Int | Long | Float | Double => unreachable!(),
                        ByteArray => array::<u8>(input)?,
                        IntArray => array::<i32>(input)?,
                        LongArray => array::<i64>(input)?,
                        List => todo!(),
                        Compound => todo!(),
                        Str => {
                                let (input, len) = big::u16(input)?;
                                let len = len as usize;
                                take(len).map(|s| (len, s)).parse(input)?
                        }
                };

                Ok((
                        input,
                        RawNbt {
                                tag,
                                name,
                                length,
                                payload,
                        },
                ))
        }
}

fn array<'input, 'a, T>(
        input: Input<'input, &'a [u8]>,
) -> IResult<'input, &'a [u8], SimpleExtras<&'a [u8]>, (usize, Vec<u8>)> {
        let (input, len) = big::i32.map(|i| i as usize).parse(input)?;
        take(len * size_of::<T>())(input).map(|bytes| (len, bytes))
}

fn main() {}

const TEST_NBT: &'static u8 = &[8];
