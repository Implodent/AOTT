use std::marker::PhantomData;

use crate::input::SliceInput;

use super::*;

pub struct MapExtra<'input, 'parse, I: InputType, E: ParserExtras<I>> {
        pub(crate) start: I::Offset,
        #[doc(hidden)]
        pub input: &'input mut Input<'parse, I, E>,
}

impl<'input, 'parse, I: InputType, E: ParserExtras<I>> MapExtra<'input, 'parse, I, E> {
        pub fn span(&self) -> I::Span {
                self.input.span_since(self.start)
        }

        pub fn slice(&self) -> I::Slice
        where
                I: SliceInput<'parse>,
        {
                self.input.input.slice(self.span())
        }

        pub fn context(&self) -> &E::Context {
                self.input.context()
        }
}

pub struct Or<A, B>(pub(crate) A, pub(crate) B);
impl<A, B, I: InputType, O, E: ParserExtras<I>> Parser<I, O, E> for Or<A, B>
where
        A: Parser<I, O, E>,
        B: Parser<I, O, E>,
{
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                let befunge = input.save();
                self.0.check_with(input).or_else(|_| {
                        input.rewind(befunge);
                        self.1.check_with(input)
                })
        }

        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O, E> {
                let befunge = input.save();
                self.0.parse_with(input).or_else(|_| {
                        input.rewind(befunge);
                        self.1.parse_with(input)
                })
        }
}

pub struct Map<A, O, F, U>(
        pub(crate) A,
        pub(crate) PhantomData<O>,
        pub(crate) F,
        pub(crate) PhantomData<U>,
);
impl<I: InputType, O, E: ParserExtras<I>, U, A: Parser<I, O, E>, F: Fn(O) -> U> Parser<I, U, E>
        for Map<A, O, F, U>
{
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.0.check_with(input)
        }
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, U, E> {
                self.0.parse_with(input).map(&self.2)
        }
}

pub struct To<A, O, U>(pub(crate) A, pub(crate) U, pub(crate) PhantomData<O>);
impl<I: InputType, O, E: ParserExtras<I>, U: Clone, A: Parser<I, O, E>> Parser<I, U, E>
        for To<A, O, U>
{
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.0.check_with(input)
        }
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, U, E> {
                self.0.check_with(input).map(|_| self.1.clone())
        }
}

pub struct TryMapWith<A, F, O, U>(
        pub(crate) A,
        pub(crate) F,
        pub(crate) PhantomData<O>,
        pub(crate) PhantomData<U>,
);

impl<
                I: InputType,
                O,
                E: ParserExtras<I>,
                U,
                F: for<'input, 'parse> Fn(
                        O,
                        &mut MapExtra<'input, 'parse, I, E>,
                ) -> Result<U, E::Error>,
                A: Parser<I, O, E>,
        > Parser<I, U, E> for TryMapWith<A, F, O, U>
{
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                let befunge = input.offset;
                self.0.parse_with(input).and_then(|thing| {
                        self.1(
                                thing,
                                &mut MapExtra {
                                        start: befunge,
                                        input,
                                },
                        )
                        .map(|_| {})
                })
        }

        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, U, E> {
                let befunge = input.offset;
                self.0.parse_with(input).and_then(|thing| {
                        self.1(
                                thing,
                                &mut MapExtra {
                                        start: befunge,
                                        input,
                                },
                        )
                })
        }
}
