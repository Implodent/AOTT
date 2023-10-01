use crate::error::Error;
use crate::input::InputType;
use crate::parser::ParserExtras;
use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::ops::Range;

#[derive(Default, Clone, Copy, Debug)]
pub struct Err<I: InputType, E: Error<I> = Simple<<I as InputType>::Token>>(
        PhantomData<I>,
        PhantomData<E>,
);

impl<I: InputType, E: Error<I>> ParserExtras<I> for Err<I, E> {
        type Error = E;
        type Context = ();
}

#[cfg(feature = "std")]
impl<Item: Debug + Display + 'static> std::error::Error for Simple<Item> {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                None
        }
}
#[cfg(all(feature = "nightly", not(feature = "std")))]
impl<Item: Debug + 'static> core::error::Error for Simple<Item> {
        fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                None
        }
}

impl<Item: Clone, I: InputType<Token = Item>> Error<I> for Simple<Item> {
        fn filter_failed(
                span: Range<usize>,
                location: &'static core::panic::Location<'static>,
                token: <I as InputType>::Token,
        ) -> Self {
                Self::FilterFailed {
                        span,
                        location,
                        token,
                }
        }
        fn expected_eof_found(span: Range<usize>, found: Item) -> Self {
                Self::ExpectedEOF { found, span }
        }
        fn expected_token_found(
                span: Range<usize>,
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
                span: Range<usize>,
                expected: Option<Vec<<I as InputType>::Token>>,
        ) -> Self {
                Self::UnexpectedEOF { span, expected }
        }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Simple<Item> {
        ExpectedEOF {
                found: Item,
                span: Range<usize>,
        },
        UnexpectedEOF {
                span: Range<usize>,
                expected: Option<Vec<Item>>,
        },
        ExpectedTokenFound {
                span: Range<usize>,
                expected: Vec<Item>,
                found: Item,
        },
        FilterFailed {
                span: Range<usize>,
                location: &'static core::panic::Location<'static>,
                token: Item,
        },
}

impl<Item: Debug> Display for Simple<Item> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                        Self::ExpectedEOF { found, span: _ } => {
                                write!(f, "expected end of file, found {found:?}")
                        }
                        Self::ExpectedTokenFound {
                                expected,
                                found,
                                span: _,
                        } => {
                                write!(f, "expected {expected:?}, found {found:?}")
                        }
                        Self::UnexpectedEOF { expected, span: _ } => match expected {
                                Some(expected) => {
                                        write!(f, "unexpected end of file, expected {expected:?}")
                                }
                                None => write!(f, "unexpected end of file"),
                        },
                        Self::FilterFailed {
                                span,
                                location,
                                token,
                        } => write!(
                                f,
                                "filter failed at {}..{} in {location}, with token {token:?}",
                                span.start, span.end
                        ),
                }
        }
}
