//! Utilities for trying to recover from errors and parse as far as possible.
//!
//! The thing you're gonna be using the most - [`Strategy`]-ies, they define how you try to recover from errors.
//! You could use a parser to recover - that's a strategy - [`via_parser`];
//! you could do nothing - that's also a strategy - [`noop`].
//!
//! Currently, there aren't many useful strategies, no, not even a lot of strategies at all; and this whole module is at its infancy and things could change, and they *would* change,
//! and if you use this before it's stable, you take the responsibility for using it, and take on the burden of adapting your code to new changes when you update.
//! It's not as unstable as other, smaller features, but it isn't stable, tested and ready to use either.
use crate::{
        go_extra,
        input::{Input, InputType},
        parser::Mode,
        prelude::{Parser, ParserExtras},
};

pub trait Strategy<I: InputType, O, E: ParserExtras<I>> {
        /// Attempt to recover from a parsing failure.
        ///
        /// **Note** A strategy should not care about properly rewinding the input,
        /// because that should be done by outer parsers (i.e. [`RecoverWith`])
        /// or anything that invokes the strategy on said input.
        fn recover<M: Mode, P: Parser<I, O, E>>(
                &self,
                input: &mut Input<I, E>,
                parser: &P,
                error: E::Error,
        ) -> Result<M::Output<O>, E::Error>;
}

#[derive(Copy, Clone)]
pub struct ViaParser<A>(A);

impl<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>> Strategy<I, O, E> for ViaParser<A> {
        fn recover<M: Mode, P: Parser<I, O, E>>(
                &self,
                input: &mut Input<I, E>,
                _parser: &P,
                error: E::Error,
        ) -> Result<M::Output<O>, E::Error> {
                // emit the error as a secondary if the parser succeeds at recovering,
                // else return the initial error
                match self.0.go::<M>(input) {
                        Ok(out) => {
                                input.errors.emit(input.offset, error);
                                Ok(out)
                        }
                        Err(_) => Err(error),
                }
        }
}

/// Try to recover from an error with this parser.
pub fn via_parser<A>(parser: A) -> ViaParser<A> {
        ViaParser(parser)
}

#[derive(Default, Copy, Clone)]
pub struct NoOp;

impl<I: InputType, O, E: ParserExtras<I>> Strategy<I, O, E> for NoOp {
        fn recover<M: Mode, P: Parser<I, O, E>>(
                &self,
                _input: &mut Input<I, E>,
                _parser: &P,
                error: E::Error,
        ) -> Result<M::Output<O>, E::Error> {
                Err(error)
        }
}

/// The recovery strategy that does nothing.
pub fn noop() -> NoOp {
        NoOp
}

#[derive(Copy, Clone)]
pub struct RecoverWith<A, S> {
        pub(crate) parser: A,
        pub(crate) strategy: S,
}

impl<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>, S: Strategy<I, O, E>> Parser<I, O, E>
        for RecoverWith<A, S>
{
        fn go<M: Mode>(&self, input: &mut Input<I, E>) -> Result<M::Output<O>, E::Error> {
                let before = input.save();

                match self.parser.go::<M>(input) {
                        Ok(out) => Ok(out),
                        Err(error) => {
                                input.rewind(before);

                                match self.strategy.recover::<M, A>(input, &self.parser, error) {
                                        Ok(out) => Ok(out),
                                        Err(rec_error) => {
                                                input.rewind(before);
                                                Err(rec_error)
                                        }
                                }
                        }
                }
        }

        go_extra!(O);
}
