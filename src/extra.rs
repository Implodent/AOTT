use crate::error::Error;
use crate::input::{InputType, Span};
use crate::parser::ParserExtras;
#[cfg(feature = "builtin-text")]
use crate::text::Char;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Default, Clone, Copy, Debug)]
pub struct Err<I: InputType, E: Error<I> = Simple<I>>(PhantomData<I>, PhantomData<E>);

impl<I: InputType, E: Error<I>> ParserExtras<I> for Err<I, E> {
        type Error = E;
        type Context = ();
}

macro_rules! simple {
        ($bound:tt) => {
                #[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
                pub enum Simple<I: InputType> where I::Token: $bound {
                        #[error(
                                "expected end of file at {}..{}, but found {found:?}",
                                .span.start(),
                                .span.end()
                        )]
                        ExpectedEOF { found: I::Token, span: I::Span },
                        #[error(
                                "unexpected end of file at {}..{}, expected {expected:?}",
                                .span.start(),
                                .span.end()
                        )]
                        UnexpectedEOF {
                                span: I::Span,
                                expected: Option<Vec<I::Token>>,
                        },
                        #[error(
                                "expected {expected:?} at {}..{}, but found {found:?}",
                                .span.start(),
                                .span.end()
                        )]
                        ExpectedTokenFound {
                                span: I::Span,
                                expected: Vec<I::Token>,
                                found: I::Token,
                        },
                        #[cfg(feature = "builtin-text")]
                        #[error(
                                "{error} at {}..{}, last token was {last_token:?}",
                                .span.start(),
                                .span.end()
                        )]
                        Text {
                                span: I::Span,
                                error: crate::text::CharLabel<I::Token>,
                                last_token: Option<I::Token>,
                        },
                        #[error(
                                "{label} at {}..{}, last token was {last_token:?}",
                                .span.start(),
                                .span.end()
                        )]
                        Sequence {
                                span: I::Span,
                                label: crate::primitive::SeqLabel<I::Token>,
                                last_token: Option<I::Token>,
                        },
                        #[error(
                                "{} at {}..{}, last token was {last_token:?}",
                                .label.0,
                                .span.start(),
                                .span.end()
                        )]
                        Filtering {
                                span: I::Span,
                                label: crate::error::Filtering,
                                last_token: Option<I::Token>,
                        },
                }

                impl<I: InputType> Error<I>
                        for Simple<I> where I::Token: $bound
                {
                        fn expected_eof_found(span: I::Span, found: I::Token) -> Self {
                                Self::ExpectedEOF { found, span }
                        }
                        fn expected_token_found(
                                span: I::Span,
                                expected: Vec<<I as InputType>::Token>,
                                found: <I as InputType>::Token,
                        ) -> Self {
                                Self::ExpectedTokenFound {
                                        span,
                                        expected,
                                        found,
                                }
                        }
                        fn unexpected_eof(
                                span: I::Span,
                                expected: Option<Vec<<I as InputType>::Token>>,
                        ) -> Self {
                                Self::UnexpectedEOF { span, expected }
                        }
                }

                impl<I: InputType>
                        crate::error::LabelError<I, crate::primitive::SeqLabel<I::Token>> for Simple<I>where I::Token: $bound
                {
                        fn from_label(
                                span: I::Span,
                                label: crate::primitive::SeqLabel<I::Token>,
                                last_token: Option<I::Token>,
                        ) -> Self {
                                Self::Sequence {
                                        span,
                                        label,
                                        last_token,
                                }
                        }
                }

                impl<I: InputType>
                        crate::error::LabelError<I, crate::error::Filtering> for Simple<I>where I::Token: $bound
                {
                        fn from_label(
                                span: I::Span,
                                label: crate::error::Filtering,
                                last_token: Option<I::Token>,
                        ) -> Self {
                                Self::Filtering {
                                        span,
                                        label,
                                        last_token,
                                }
                        }
                }

                #[cfg(feature = "builtin-text")]
                impl<I: InputType>
                        crate::error::LabelError<I, crate::text::CharLabel<I::Token>> for Simple<I> where I::Token: Char
                {
                        fn from_label(
                                span: I::Span,
                                error: crate::text::CharLabel<I::Token>,
                                last_token: Option<I::Token>,
                        ) -> Self {
                                Self::Text {
                                        span,
                                        error,
                                        last_token,
                                }
                        }
                }
        };
}

#[cfg(feature = "builtin-text")]
simple!(Char);

#[cfg(not(feature = "builtin-text"))]
simple!(Nothing);

#[cfg(not(feature = "builtin-text"))]
#[doc(hidden)]
pub trait Nothing {}

#[cfg(not(feature = "builtin-text"))]
impl<T: ?Sized> Nothing for T {}
