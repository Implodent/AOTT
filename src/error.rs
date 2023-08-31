use core::ops::Range;

use num_traits::Zero;

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

// #[derive(Debug)]
pub struct ParseResult<'parse, I: InputType, O, E: ParserExtras<I>> {
        pub input: Input<'parse, I, E>,
        pub output: Option<O>,
        pub errors: Vec<E::Error>,
}

impl<'parse, I: InputType, O, E: ParserExtras<I>> ParseResult<'parse, I, O, E> {
        pub fn single(result: IResult<'parse, I, E, O>) -> Self {
                match result {
                        Ok((input, ok)) => Self {
                                input,
                                output: Some(ok),
                                errors: vec![],
                        },
                        Err((input, err)) => Self {
                                input,
                                output: None,
                                errors: vec![err],
                        },
                }
        }

        pub fn into_result(mut self) -> Result<O, E::Error> {
                self.output
                        .take()
                        .ok_or_else(|| self.errors.pop().expect("huh"))
        }
        pub fn has_errors(&self) -> bool {
                !self.errors.is_empty()
        }
        pub fn has_advanced(&self) -> bool {
                !self.input.offset.is_zero()
        }
}

pub type IResult<'parse, I, E, O> =
        Result<(Input<'parse, I, E>, O), (Input<'parse, I, E>, <E as ParserExtras<I>>::Error)>;
