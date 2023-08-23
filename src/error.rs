use core::ops::Range;

use crate::{
        input::{Input, InputType},
        parser::ParserExtras,
        MaybeRef,
};

pub trait Span {
        type Offset: From<usize>;

        // fn new(context: Self::Context, offset: Range<Self::Offset>) -> Self;
        fn new(offset: Range<Self::Offset>) -> Self;
        fn new_usize<T: Into<usize>>(offset: Range<T>) -> Self
        where
                Self: Sized,
        {
                Self::new(
                        (Into::<usize>::into(offset.start).into())
                                ..(Into::<usize>::into(offset.end).into()),
                )
        }
        // fn context(&self) -> Self::Context;
        fn start(&self) -> Self::Offset;
        fn end(&self) -> Self::Offset;
}

impl<T: Clone + From<usize>> Span for Range<T> {
        type Offset = T;
        // type Context = ();
        // fn new((): Self::Context, offset: Range<Self::Offset>) -> Self {
        //         offset
        // }
        fn new(offset: Range<Self::Offset>) -> Self {
                offset
        }
        // fn context(&self) -> Self::Context {}
        fn start(&self) -> Self::Offset {
                self.start.clone()
        }
        fn end(&self) -> Self::Offset {
                self.end.clone()
        }
}

// impl<T: Clone, C: Clone> Span for (C, Range<T>) {
//         type Context = C;
//         type Offset = T;
//         fn new(context: Self::Context, offset: Range<Self::Offset>) -> Self {
//                 (context, offset)
//         }
//         fn context(&self) -> Self::Context {
//                 self.0.clone()
//         }
//         fn start(&self) -> Self::Offset {
//                 self.1.start.clone()
//         }
//         fn end(&self) -> Self::Offset {
//                 self.1.end.clone()
//         }
// }

pub trait Error<I: InputType>: Sized {
        type Span: Span;

        /// expected end of input at `span`, found `found`
        fn expected_eof_found(span: Self::Span, found: MaybeRef<'_, I::Token>) -> Self;
        /// unexpected end of input at `span`
        fn unexpected_eof(span: Self::Span, expected: Option<Vec<I::Token>>) -> Self;
        /// expected tokens (any of) `expected`, found `found`
        fn expected_token_found(
                span: Self::Span,
                expected: Vec<I::Token>,
                found: MaybeRef<'_, I::Token>,
        ) -> Self;
        fn expected_token_found_or_eof(
                span: Self::Span,
                expected: Vec<I::Token>,
                found: Option<MaybeRef<'_, I::Token>>,
        ) -> Self {
                match found {
                        Some(found) => Self::expected_token_found(span, expected, found),
                        None => Self::unexpected_eof(span, Some(expected)),
                }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simple<Item> {
        pub span: Range<usize>,
        pub reason: SimpleReason<Item>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult<I, O, E> {
        pub input: I,
        pub output: Option<O>,
        pub errors: Vec<E>,
}

impl<I, O, E> ParseResult<I, O, E> {
        pub fn single((input, result): (I, Result<O, E>)) -> Self {
                match result {
                        Ok(ok) => Self {
                                input,
                                output: Some(ok),
                                errors: vec![],
                        },
                        Err(err) => Self {
                                input,
                                output: None,
                                errors: vec![err],
                        },
                }
        }
}

pub type IResult<'parse, I, E, O> =
        Result<(Input<'parse, I, E>, O), (Input<'parse, I, E>, <E as ParserExtras<I>>::Error)>;
