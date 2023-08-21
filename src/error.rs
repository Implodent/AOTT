use core::ops::Range;

use crate::{input::Input, MaybeRef};

pub trait Span {
        type Context;
        type Offset;

        fn new(context: Self::Context, offset: Range<Self::Offset>) -> Self;
        fn context(&self) -> Self::Context;
        fn start(&self) -> Self::Offset;
        fn end(&self) -> Self::Offset;
}

impl<T: Clone> Span for Range<T> {
        type Offset = T;
        type Context = ();
        fn new(_: Self::Context, offset: Range<Self::Offset>) -> Self {
                offset
        }
        fn context(&self) -> Self::Context {}
        fn start(&self) -> Self::Offset {
                self.start.clone()
        }
        fn end(&self) -> Self::Offset {
                self.end.clone()
        }
}

impl<T: Clone, C: Clone> Span for (C, Range<T>) {
        type Context = C;
        type Offset = T;
        fn new(context: Self::Context, offset: Range<Self::Offset>) -> Self {
                (context, offset)
        }
        fn context(&self) -> Self::Context {
                self.0.clone()
        }
        fn start(&self) -> Self::Offset {
                self.1.start.clone()
        }
        fn end(&self) -> Self::Offset {
                self.1.end.clone()
        }
}

pub trait Error<I: Input>: Sized {
        type Span: Span;

        /// expected end of input at `span`, found `found`
        fn expected_eof_found<'a>(span: Self::Span, found: Option<MaybeRef<'a, I::Token>>) -> Self;
        /// unexpected end of input at `span`
        fn unexpected_eof(span: Self::Span) -> Self;
        /// expected tokens (any of) `expected`, found `found`
        fn expected_token_found<'a>(
                span: Self::Span,
                expected: Vec<I::Token>,
                found: Option<MaybeRef<'a, I::Token>>,
        ) -> Self;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simple<Item> {
        pub span: Range<usize>,
        pub reason: SimpleReason<Item>,
}

impl<Item, I: Input<Token = Item>> Error<I> for Simple<Item> {
        type Span = Range<usize>;

        fn expected_eof_found(span: Self::Span, found: Item) -> Self {
                Self {
                        span,
                        reason: SimpleReason::ExpectedEOF { found },
                }
        }
        fn expected_token_found(span: Self::Span, expected: Vec<Item>, found: Item) -> Self {
                Self {
                        span,
                        reason: SimpleReason::ExpectedTokenFound { expected, found },
                }
        }
        fn unexpected_eof(span: Self::Span) -> Self {
                Self {
                        span,
                        reason: SimpleReason::UnexpectedEOF,
                }
        }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleReason<Item> {
        ExpectedEOF { found: Item },
        UnexpectedEOF,
        ExpectedTokenFound { expected: Vec<Item>, found: Item },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult<I: Input, O, E: Error<I>> {
        pub input: I,
        pub output: Option<O>,
        pub errors: Vec<E>,
}
