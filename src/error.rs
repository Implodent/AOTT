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

#[derive(Copy, Clone, Debug, PartialEq, Eq, derive_more::Display)]
pub enum BuiltinLabel {
        #[display(
                fmt = "not enough elements: expected {expected_amount} elements, but found {found_amount}"
        )]
        NotEnoughElements {
                expected_amount: usize,
                found_amount: usize,
        },
}

pub trait LabelError<I: InputType, L>: Sized {
        fn from_label(span: Range<usize>, label: L, last_token: Option<I::Token>) -> Self;
}

/// A trait for AOTT errors. It's done like this so general things that use filter (and similar) would use this onnnne
pub trait Error<I: InputType>: FundamentalError<I> + LabelError<I, BuiltinLabel> {}
impl<I: InputType, E: FundamentalError<I> + LabelError<I, BuiltinLabel>> Error<I> for E {}

pub trait LabelWith<I: InputType, E: LabelError<I, Self>>: Sized {
        fn error(self, span: Range<usize>, last_token: Option<I::Token>) -> E;
}

impl<I: InputType, L, E: LabelError<I, L>> LabelWith<I, E> for L {
        fn error(self, span: Range<usize>, last_token: Option<<I as InputType>::Token>) -> E {
                E::from_label(span, self, last_token)
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

pub type PResult<O, E = crate::extra::Err<I>> =
        core::result::Result<O, <E as ParserExtras<I>>::Error>;

/// Implement `LabelError<I, Filtering>` to use `filter*` with your error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filtering(pub Cow<'static, str>);

pub trait Invert<T, E> {
        type This<T_, E_>;

        fn invert(
                self,
                err_to_ok: impl FnOnce(E) -> T,
                ok_to_err: impl FnOnce(T) -> E,
        ) -> Self::This<E, T>;
}

impl<T, E> Invert<T, E> for Result<T, E> {
        type This<T_, E_> = Result<T_, E_>;

        fn invert(
                self,
                err_to_ok: impl FnOnce(E) -> T,
                ok_to_err: impl FnOnce(T) -> E,
        ) -> Self::This<T, E> {
                match self {
                        Ok(ok) => Err(ok_to_err(ok)),
                        Err(err) => Ok(err_to_ok(err)),
                }
        }
}
