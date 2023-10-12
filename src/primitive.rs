// just, filter, end, nothing, one_of, none_of, separated_by, filter_map,
// select!, take, take_while, take_while_bounded

use core::{borrow::Borrow, ops::Range};

use crate::{
        container::OrderedSeq,
        error::{Error, FundamentalError},
        input::{Input, InputType},
        parser::{Emit, Mode, Parser, ParserExtras},
        pfn_type, EmptyPhantom, PResult,
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
pub use just::*;
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
pub fn any<I: InputType, E: ParserExtras<I>>(input: I) -> I::Token {
        input.next()
}

#[derive(Copy, Clone)]
pub struct Ignored<A, OA>(pub(crate) A, pub(crate) EmptyPhantom<OA>);
impl<I: InputType, E: ParserExtras<I>, A: Parser<I, OA, E>, OA> Parser<I, (), E>
        for Ignored<A, OA>
{
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.check_with(input)
        }
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.0.check_with(input)
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
/// let parser = just("eof").then_ignore(end::<_, extra::Err<_>>);
/// assert_eq!(parser.parse(input), Ok("eof"));
/// ```
pub fn end<I: InputType, E: ParserExtras<I>>(input: I) {
        let offset = input.offset;
        match input.next_or_none() {
                Some(found) => {
                        let err = FundamentalError::expected_eof_found(
                                input.span_since(offset),
                                found,
                        );
                        Err(err)
                }
                None => Ok(()),
        }
}

/// This function makes a parser optional -
/// if it returns an error, this parser succeeds
/// and just returns None as the output.
/// # Example
/// ```
/// # use aott::prelude::*;
/// let parser = maybe::<&str, extra::Err<&str>, _, _>(just("domatch"));
/// let input = "dontmatch";
/// assert_eq!(parser.parse(input), Ok(None));
/// ```
pub fn maybe<I: InputType, E: ParserExtras<I>, O, A: Parser<I, O, E>>(parser: A) -> Maybe<A> {
        Maybe(parser)
}

/// A parser that skips all tokens while `filter` returns true.
/// When it returns [`false`], the cycle stops and the function returns.
///
/// **Note** This parser does not allocate. It uses the [`Input::skip_while`] function, which does not allocate. You are safe to use this in `check_with` functions.
///
/// # Example
/// ```ignore
/// // snipped from text module
/// let sw = skip_while(Char::is_whitespace);
/// sw(input)?;
/// let output = self.0.parse_with(input)?;
/// sw(input)?;
/// Ok(output)
/// ```
pub fn skip_while<I: InputType, E: ParserExtras<I>, F: Fn(&I::Token) -> bool>(
        filter: F,
) -> pfn_type!(I, (), E) {
        move |input| {
                input.skip_while(&filter);
                Ok(())
        }
}

#[derive(Copy, Clone)]
pub struct Maybe<A>(pub(crate) A);

impl<I: InputType, E: ParserExtras<I>, O, A: Parser<I, O, E>> Parser<I, Option<O>, E> for Maybe<A> {
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, Option<O>, E> {
                let befunge = input.save();
                Ok(self.0.parse_with(input).map_or_else(
                        |_| {
                                input.rewind(befunge);
                                None
                        },
                        Some,
                ))
        }
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                let befunge = input.save();
                self.0.check_with(input)
                        .unwrap_or_else(|_| input.rewind(befunge));
                Ok(())
        }
}
