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
pub fn end<I: InputType, E: ParserExtras<I>>(mut input: I) {
        let offset = input.offset;
        match input.next() {
                Some(found) => {
                        let err = Error::expected_eof_found(
                                Span::new_usize(input.span_since(offset)),
                                crate::Maybe::Val(found),
                        );
                        Err((input, err))
                }
                None => Ok((input, ())),
        }
}

#[cfg(test)]
mod test {
        use crate::{
                input::InputOwned,
                parser::{Parser, SimpleExtras},
                select,
                stream::Stream,
        };

        use super::*;

        #[test]
        fn just_parses_a() {
                let mut input =
                        InputOwned::<&'static str, SimpleExtras<&'static str>>::from_input("abc");
                {
                        let inp = input.as_ref_at_zero();
                        let parser = just("ab");

                        let result = parser.parse(inp);
                        assert!(result.is_ok());
                        let (input, output) = result.expect("fail");
                        assert_eq!(&input.input[input.offset..], "c");
                        assert_eq!(output, "ab");
                }
        }

        #[test]
        fn tuple_parser_a_b() {
                let mut input =
                        InputOwned::<&'static str, SimpleExtras<&'static str>>::from_input("abcd");
                {
                        let inp = input.as_ref_at_zero();
                        let parser = tuple((just("ab"), just("cd")));

                        let result = parser.parse(inp);
                        assert!(result.is_ok());
                        let (input, output) = result.expect("fail");
                        assert_eq!(&input.input[input.offset..], "");
                        assert_eq!(output, ("ab", "cd"));
                }
        }

        #[test]
        fn choice_chooses_ab() {
                let mut input =
                        InputOwned::<&'static str, SimpleExtras<&'static str>>::from_input("abcd");
                {
                        let inp = input.as_ref_at_zero();
                        let parser = choice((just("ab"), just("cd")));

                        let result = parser.parse(inp);
                        assert!(result.is_ok());
                        let (input, output) = result.expect("fail");
                        assert_eq!(&input.input[input.offset..], "cd");
                        assert_eq!(output, "ab");
                }
        }

        #[test]
        fn example_if_statement() {
                #[derive(Clone, PartialEq, Eq)]
                enum Token {
                        KwIf,
                        KwTrue,
                        KwFalse,
                        OpenCurly,
                        KwPrint,
                        Str(String),
                        CloseCurly,
                }
                #[derive(Debug)]
                struct IfStatement {
                        condition: bool,
                        print: String,
                }
                type Tokens = Stream<<Vec<Token> as IntoIterator>::IntoIter>;

                fn bool() -> impl Parser<Tokens, bool, SimpleExtras<Tokens>> {
                        choice((just(Token::KwTrue).to(true), just(Token::KwFalse).to(false)))
                }
                fn if_statement(
                        inp: Input<'_, Tokens>,
                ) -> IResult<'_, Tokens, SimpleExtras<Tokens>, IfStatement> {
                        tuple((
                                just(Token::KwIf),
                                bool(),
                                just(Token::OpenCurly),
                                just(Token::KwPrint),
                                select!(Token::Str(str) => str),
                                just(Token::CloseCurly),
                        ))
                        .map(|(_, condition, _, _, print, _)| IfStatement { condition, print })
                        .parse(inp)
                }
                let tokens = vec![
                        Token::KwIf,
                        Token::KwFalse,
                        Token::OpenCurly,
                        Token::KwPrint,
                        Token::Str(String::from("hello world!")),
                        Token::CloseCurly,
                ];
                let mut input = InputOwned::from_input(Stream::from_iter(tokens));
                {
                        let inp = input.as_ref_at_zero();
                        let Ok((_, output)) = inp.parse(&if_statement) else {
                                panic!("test fail")
                        };
                        assert!(!output.condition);
                        assert_eq!(output.print, "hello world!");
                }
        }
}
