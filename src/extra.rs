use crate::error::Error;
use crate::input::InputType;
use crate::parser::ParserExtras;
use crate::MaybeRef;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simple<Item> {
        pub span: Range<usize>,
        pub reason: SimpleReason<Item>,
}

#[cfg(feature = "std")]
impl<Item: Debug + Display> std::error::Error for Simple<Item> {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                None
        }
}
#[cfg(all(feature = "nightly", not(feature = "std")))]
impl<Item: Debug> core::error::Error for Simple<Item> {
        fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                None
        }
}
impl<Item: Debug> Display for Simple<Item> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let Self { span, reason } = self;
                write!(f, "{reason} (at {}..{})", span.start, span.end)
        }
}

impl<Item: Clone, I: InputType<Token = Item>> Error<I> for Simple<Item> {
        type Span = Range<usize>;

        fn expected_eof_found(span: Self::Span, found: MaybeRef<'_, Item>) -> Self {
                Self {
                        span,
                        reason: SimpleReason::ExpectedEOF {
                                found: found.into_clone(),
                        },
                }
        }
        fn expected_token_found(
                span: Self::Span,
                expected: Vec<Item>,
                found: MaybeRef<'_, Item>,
        ) -> Self {
                Self {
                        span,
                        reason: SimpleReason::ExpectedTokenFound {
                                expected,
                                found: found.into_clone(),
                        },
                }
        }
        fn unexpected_eof(span: Self::Span, expected: Option<Vec<Item>>) -> Self {
                Self {
                        span,
                        reason: SimpleReason::UnexpectedEOF(expected),
                }
        }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleReason<Item> {
        ExpectedEOF { found: Item },
        UnexpectedEOF(Option<Vec<Item>>),
        ExpectedTokenFound { expected: Vec<Item>, found: Item },
}

impl<Item: Debug> Display for SimpleReason<Item> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                        Self::ExpectedEOF { found } => {
                                write!(f, "expected end of file, found {found:?}")
                        }
                        Self::ExpectedTokenFound { expected, found } => {
                                write!(f, "expected {expected:?}, found {found:?}")
                        }
                        Self::UnexpectedEOF(expected) => match expected {
                                Some(expected) => {
                                        write!(f, "unexpected end of file, expected {expected:?}")
                                }
                                None => write!(f, "unexpected end of file"),
                        },
                }
        }
}
