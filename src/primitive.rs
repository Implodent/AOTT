// just, filter, end, nothing, one_of, none_of, separated_by, filter_map,
// select!, take, take_while, take_while_bounded

use core::borrow::Borrow;

use crate::{
        container::OrderedSeq,
        error::{Error, Located, Span},
        explode_extra,
        input::{Input, InputType},
        parser::{Check, Emit, Mode, PResult, Parser, ParserExtras},
        IResult, MaybeRef,
};

mod choice;
mod filter;
mod just;
mod tuple;

pub use choice::*;
pub use filter::*;
pub use just::just;
pub use tuple::*;

pub fn any<I: InputType, E: ParserExtras<I>>(
        mut input: Input<'_, I, E>,
) -> IResult<'_, I, E, I::Token> {
        let befunge = input.offset;
        match input.next() {
                Some(token) => Ok((input, token)),
                None => {
                        let err = Error::unexpected_eof(
                                Span::new_usize(input.span_since(befunge)),
                                None,
                        );
                        Err((input, err))
                }
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
