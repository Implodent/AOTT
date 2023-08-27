use core::marker::PhantomData;

use super::*;

pub struct Or<A, B>(pub(crate) A, pub(crate) B);

impl<A, B, I: InputType, O, E: ParserExtras<I>> Parser<I, O, E> for Or<A, B>
where
        A: Parser<I, O, E>,
        B: Parser<I, O, E>,
{
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.0.check(input)
                        .or_else(|(input, _)| self.1.check(input))
        }

        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, O> {
                self.0.parse(input)
                        .or_else(|(input, _)| self.1.parse(input))
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
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.0.check(input)
        }
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, U> {
                self.0.parse(input)
                        .map(|(input, thing)| (input, self.2(thing)))
        }
}

pub struct To<A, O, U>(pub(crate) A, pub(crate) U, pub(crate) PhantomData<O>);
impl<I: InputType, O, E: ParserExtras<I>, U: Clone, A: Parser<I, O, E>> Parser<I, U, E>
        for To<A, O, U>
{
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.0.check(input)
        }
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, U> {
                self.0.check(input)
                        .map(|(input, ())| (input, self.1.clone()))
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
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.0.parse(input)
                        .and_then(|(input, thing)| match self.1(thing) {
                                Ok(_) => Ok((input, ())),
                                Err(e) => Err((input, e)),
                        })
        }

        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, U> {
                self.0.parse(input)
                        .and_then(|(input, thing)| match self.1(thing) {
                                Ok(ok) => Ok((input, ok)),
                                Err(e) => Err((input, e)),
                        })
        }
}
