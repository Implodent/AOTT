// just, filter, end, nothing, one_of, none_of, separated_by, filter_map,
// select!, take, take_while, take_while_bounded

use core::{borrow::Borrow, ops::Range};

use crate::{
        container::OrderedSeq,
        error::{Error, Span},
        input::{Input, InputType},
        parser::{Emit, Mode, Parser, ParserExtras},
        EmptyPhantom, IResult, MaybeRef,
};

mod choice;
mod filter;
mod just;
mod map;
mod recursive;
mod sequence;
mod take;
mod tuple;

use aott_derive::parser;
pub use choice::*;
pub use filter::*;
pub use just::just;
pub use map::*;
pub use recursive::*;
pub use sequence::*;
pub use take::*;
pub use tuple::*;

#[parser(extras = E)]
/// A parser that accepts any input token,
/// (but not the end of input, for which it returns an error)
/// and returns it as-is.
/// # Errors
/// This function returns an error if end of file is reached.
pub fn any<I: InputType, E: ParserExtras<I>>(mut input: I) -> I::Token {
        match input.next_or_eof() {
                Ok(ok) => Ok((input, ok)),
                Err(err) => Err((input, err)),
        }
}

#[derive(Copy, Clone)]
pub struct Ignored<A, OA>(pub(crate) A, pub(crate) EmptyPhantom<OA>);
impl<I: InputType, E: ParserExtras<I>, A: Parser<I, OA, E>, OA> Parser<I, (), E>
        for Ignored<A, OA>
{
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.0.check(input)
        }
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.0.check(input)
        }
}

#[parser(extras = E)]
/// A parser that accepts only end of input.
/// The output type of this parser is `()`.
///
/// # Errors
/// This function returns an error if end of input was not reached.
///
/// # Example
/// ```
/// # use aott::prelude::*;
/// let input = "eof";
/// let (_, (output, ())) = (just("eof").then(end)).parse(Input::<&str, SimpleExtras<&str>>::new(&input)).unwrap();
/// assert_eq!(output, "eof");
/// ```
pub fn end<I: InputType, E: ParserExtras<I>>(mut input: I) {
        let offset = input.offset;
        match input.next() {
                Some(found) => {
                        let err = Error::expected_eof_found(
                                Span::new_usize(input.span_since(offset)),
                                crate::MaybeDeref::Val(found),
                        );
                        Err((input, err))
                }
                None => Ok((input, ())),
        }
}

/// This function makes a parser optional -
/// if it returns an error, this parser succeeds
/// and just returns None as the output.
/// # Example
/// ```
/// # use aott::prelude::*;
/// let input = "dontmatch";
/// let (_, output) = (maybe::<&str, SimpleExtras<&str>, _, _>(just("domatch"))).parse_from(&input).unwrap();
/// assert_eq!(output, None);
/// ```
pub fn maybe<I: InputType, E: ParserExtras<I>, O, A: Parser<I, O, E>>(parser: A) -> Maybe<A> {
        Maybe(parser)
}

#[derive(Copy, Clone)]
pub struct Maybe<A>(pub(crate) A);

impl<I: InputType, E: ParserExtras<I>, O, A: Parser<I, O, E>> Parser<I, Option<O>, E> for Maybe<A> {
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, Option<O>> {
                Ok(self.0.parse(input).map_or_else(
                        |(input, _)| (input, None),
                        |(input, thing)| (input, Some(thing)),
                ))
        }
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                Ok((input, ()))
        }
}
