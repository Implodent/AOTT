use alloc::borrow::Cow;

use crate::{input::InputType, parser::ParserExtras};

pub trait Error<I: InputType>: Sized {
        /// expected end of input at `span`, found `found`
        fn expected_eof_found(span: I::Span, found: I::Token) -> Self;

        /// unexpected end of input at `span`
        fn unexpected_eof(span: I::Span, expected: Option<Vec<I::Token>>) -> Self;

        /// expected tokens (any of) `expected`, found `found`
        fn expected_token_found(span: I::Span, expected: Vec<I::Token>, found: I::Token) -> Self;

        fn expected_token_found_or_eof(
                span: I::Span,
                expected: Vec<I::Token>,
                found: Option<I::Token>,
        ) -> Self {
                match found {
                        Some(found) => Self::expected_token_found(span, expected, found),
                        None => Self::unexpected_eof(span, Some(expected)),
                }
        }
}

pub trait LabelError<I: InputType, L>: Sized {
        fn from_label(span: I::Span, label: L, last_token: Option<I::Token>) -> Self;
}

#[derive(Clone, Debug)]
pub struct Located<L, E> {
        pub pos: L,
        pub err: E,
}

impl<L, E> Located<L, E> {
        #[inline]
        pub fn at(pos: L, err: E) -> Self {
                Self { pos, err }
        }
}

pub type PResult<I, O, E> = Result<O, <E as ParserExtras<I>>::Error>;

/// Implement `LabelError<I, Filtering>` to use `filter*` with your error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filtering(pub Cow<'static, str>);
