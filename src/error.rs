use core::ops::Range;

use crate::{input::InputType, parser::ParserExtras};

pub trait Error<I: InputType>: Sized {
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

        /// a filter failed at `span` in `location`, the token that did not pass was `token`
        fn filter_failed(
                span: Range<usize>,
                location: &'static core::panic::Location<'static>,
                token: I::Token,
        ) -> Self;

        fn not_enough_elements(
                span: Range<usize>,
                found: usize,
                expected: usize,
                last_token: Option<I::Token>,
        ) -> Self {
                Self::unexpected_eof(span, None)
        }
}

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
