use core::ops::Range;

use alloc::borrow::Cow;

use crate::{input::InputType, parser::ParserExtras};

pub trait FundamentalError<I: InputType>: Sized {
        /// expected end of input at `span`, found `found`
        fn expected_eof_found(span: Range<usize>, found: I::Token) -> Self;

        /// unexpected end of input at `span`
        fn unexpected_eof(span: Range<usize>, expected: Option<Vec<I::Token>>) -> Self;

        /// expected tokens (any of) `expected`, found `found`
        fn expected_token_found(
                span: Range<usize>,
                expected: Vec<I::Token>,
                found: I::Token,
        ) -> Self;

        fn expected_token_found_or_eof(
                span: Range<usize>,
                expected: Vec<I::Token>,
                found: Option<I::Token>,
        ) -> Self {
                match found {
                        Some(found) => Self::expected_token_found(span, expected, found),
                        None => Self::unexpected_eof(span, Some(expected)),
                }
        }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BuiltinLabel<Token> {
        NotEnoughElements {
                expected_amount: usize,
                found_amount: usize,
        },
}

pub trait LabelError<I: InputType, L>: Sized {
        fn from_label(span: Range<usize>, label: L, last_token: Option<I::Token>) -> Self;
}

/// A trait for AOTT errors. It's done like this so general things that use filter (and similar) would use this onnnne
pub trait Error<I: InputType>: FundamentalError<I> + LabelError<I, BuiltinLabel<I::Token>> {}
impl<I: InputType, E: Error<I> + LabelError<I, BuiltinLabel<I::Token>>> Error<I> for E {}

#[derive(Clone, Debug)]
pub struct Located<T, E> {
        pub pos: T,
        pub err: E,
}

impl<T, E> Located<T, E> {
        #[inline]
        pub fn at(pos: T, err: E) -> Self {
                Self { pos, err }
        }
}

pub type PResult<I, O, E = crate::extra::Err<I>> = Result<O, <E as ParserExtras<I>>::Error>;
