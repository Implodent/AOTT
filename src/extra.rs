use crate::error::Error;
use crate::input::InputType;
use crate::parser::ParserExtras;
use crate::MaybeRef;
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
