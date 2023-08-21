use core::marker::PhantomData;

use crate::{error::*, input::Input};

pub trait Parser<I: Input, O, E: ParserExtras<I>> {
        fn parse(&self, input: I) -> ParseResult<I, O, <E as ParserExtras<I>>::Error>
        where
                E::Context: Default,
        {
                self.parse_with_extras(input, E::Context::default())
        }
        fn parse_with_extras(
                &self,
                input: I,
                context: E::Context,
        ) -> ParseResult<I, O, <E as ParserExtras<I>>::Error>;
}

impl<
                I: Input,
                O,
                E: ParserExtras<I>,
                // only input
                F: Fn(I) -> ParseResult<I, O, <E as ParserExtras<I>>::Error>,
        > Parser<I, O, E> for F
{
        fn parse_with_extras(
                &self,
                input: I,
                _context: E::Context,
        ) -> ParseResult<I, O, <E as ParserExtras<I>>::Error> {
                self(input)
        }
}

pub trait ParserExtras<I: Input> {
        type Error: Error<I>;
        type Context;

        // fn recover<O>(
        //     _error: Self::Error,
        //     _context: Self::Context,
        //     input: I,
        //     prev_output: Option<O>,
        //     prev_errors: Vec<Self::Error>,
        // ) -> ParseResult<I, O, Self::Error> {
        //     // default: noop
        //     ParseResult {
        //         input,
        //         output: prev_output,
        //         errors: prev_errors,
        //     }
        // }
}

#[derive(Default, Clone, Copy)]
pub struct SimpleExtras<I, E = Simple<I>>(PhantomData<I>, PhantomData<E>);

impl<I: Input, E: Error<I>> ParserExtras<I> for SimpleExtras<I, E> {
        type Error = E;
        type Context = ();
}
