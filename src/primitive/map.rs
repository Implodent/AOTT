use core::marker::PhantomData;

use super::*;

pub struct Or<A, B>(pub(crate) A, pub(crate) B);

impl<A, B, I: InputType, O, E: ParserExtras<I>> Parser<I, O, E> for Or<A, B>
where
        A: Parser<I, O, E>,
        B: Parser<I, O, E>,
{
        fn check_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                let befunge = input.save();
                self.0.check_with(input).or_else(|_| {
                        input.rewind(befunge);
                        self.1.check_with(input)
                })
        }

        fn parse_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
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
        fn check_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                self.0.check_with(input)
        }
        fn parse_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                self.0.parse_with(input).map(&self.2)
        }
}

pub struct To<A, O, U>(pub(crate) A, pub(crate) U, pub(crate) PhantomData<O>);
impl<I: InputType, O, E: ParserExtras<I>, U: Clone, A: Parser<I, O, E>> Parser<I, U, E>
        for To<A, O, U>
{
        fn check_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                self.0.check_with(input)
        }
        fn parse_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                self.0.check_with(input).map(|_| self.1.clone())
        }
}

pub struct TryMap<A, F, O, U>(
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
                F: Fn(O) -> Result<U, E::Error>,
                A: Parser<I, O, E>,
        > Parser<I, U, E> for TryMap<A, F, O, U>
{
        fn check_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                self.0.parse_with(input).and_then(&self.1).map(|_| {})
        }

        fn parse_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                self.0.parse_with(input).and_then(|thing| self.1(thing))
        }
}

pub struct TryMapWithSpan<A, F, O, U>(
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
                F: Fn(O, Range<usize>) -> Result<U, E::Error>,
                A: Parser<I, O, E>,
        > Parser<I, U, E> for TryMapWithSpan<A, F, O, U>
{
        fn check_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                let befunge = input.offset;
                self.0.parse_with(input)
                        .and_then(|thing| self.1(thing, input.span_since(befunge)).map(|_| {}))
        }

        fn parse_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                let befunge = input.offset;
                self.0.parse_with(input)
                        .and_then(|thing| self.1(thing, input.span_since(befunge)))
        }
}
