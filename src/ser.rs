//! This module contains the Serialization API -
//! an API for defining objects that can be deserialized with AOTT.
//! The Serialization API was originally created for use with `OxCraft`,
//! for easier access to deserialization primitives, and implementations for built-in types.
use crate::{
        input::{Input, InputType},
        prelude::{take_exact, Parser, ParserExtras},
        PResult,
};
use derive_more::*;

pub trait Serialize<I: InputType> {
        fn serialize_to(&self, buf: &mut I::OwnedMut);
}

pub trait Deserialize<I: InputType, E: ParserExtras<I, Context = Self::Context>> {
        #[cfg(feature = "nightly")]
        type Context = ();
        #[cfg(not(feature = "nightly"))]
        type Context;

        fn deserialize(input: &mut Input<I, E>) -> PResult<I, Self, E>
        where
                Self: Sized;
}

pub fn deserialize_from<
        I: InputType,
        E: ParserExtras<I, Context = T::Context>,
        T: Deserialize<I, E>,
>(
        input: I,
) -> PResult<I, T, E>
where
        T::Context: Default,
{
        T::deserialize.parse_with_context(input, Default::default())
}

pub trait Endian {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Big;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Little;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Native;

impl Endian for Big {}
impl Endian for Little {}
impl Endian for Native {}

/// A type for serializing and deserializing numbers from bytes.
/// # Examples
/// ```
/// # use aott::prelude::*;
/// # use aott::ser::*;
/// assert_eq!(deserialize_from::<_, extra::Err<&[u8]>, _>(&[0x63][..]), Ok(Number(99u8, Big)));
/// ```
#[derive(Copy, Clone, Deref, DerefMut, Debug, Display, PartialEq, Eq)]
#[display(bound = "N: std::fmt::Display")]
#[display(fmt = "{_0}")]
pub struct Number<N, E: Endian>(
        #[deref]
        #[deref_mut]
        pub N,
        pub E,
);

macro_rules! number_impl {
        ($($num:ty)*) => {
                $(impl<I: InputType<Token = u8>, E: ParserExtras<I, Context = ()>> Deserialize<I, E>
                        for Number<$num, Big>
                {
                        type Context = ();

                        fn deserialize(input: &mut Input<I, E>) -> PResult<I, Self, E>
                        where
                                Self: Sized,
                        {
                                Ok(Number(
                                        <$num>::from_be_bytes(
                                                take_exact::<{ std::mem::size_of::<$num>() }>()
                                                        .parse_with(input)?,
                                        ),
                                        Big,
                                ))
                        }
                }
                impl<I: InputType<Token = u8>, E: ParserExtras<I, Context = ()>> Deserialize<I, E>
                        for Number<$num, Little>
                {
                        type Context = ();

                        fn deserialize(input: &mut Input<I, E>) -> PResult<I, Self, E>
                        where
                                Self: Sized,
                        {
                                Ok(Number(
                                        <$num>::from_le_bytes(
                                                take_exact::<{ std::mem::size_of::<$num>() }>()
                                                        .parse_with(input)?,
                                        ),
                                        Little,
                                ))
                        }
                }
                impl<I: InputType<Token = u8>, E: ParserExtras<I, Context = ()>> Deserialize<I, E>
                        for Number<$num, Native>
                {
                        type Context = ();

                        fn deserialize(input: &mut Input<I, E>) -> PResult<I, Self, E>
                        where
                                Self: Sized,
                        {
                                Ok(Number(
                                        <$num>::from_ne_bytes(
                                                take_exact::<{ std::mem::size_of::<$num>() }>()
                                                        .parse_with(input)?,
                                        ),
                                        Native,
                                ))
                        }
                }
            )*
        };
}

number_impl![u8 u16 u32 u64 u128 i8 i16 i32 i64 i128];
