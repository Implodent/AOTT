// just, filter, end, nothing, one_of, none_of, separated_by, filter_map,
// select!, take, take_while, take_while_bounded

use core::borrow::Borrow;

use crate::{
        container::OrderedSeq,
        error::{Error, Span},
        input::{Input, InputType},
        parser::{Parser, ParserExtras},
        IResult, MaybeRef,
};

pub fn just<'a, I: InputType, T: OrderedSeq<'a, I::Token> + Clone, E: ParserExtras<I>>(
        seq: T,
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, T>
where
        I::Token: Eq + Clone + 'static,
{
        move |mut input| {
                if let Some(err) = seq.seq_iter().find_map(|next| {
                        let befunge = input.offset;
                        let next = T::to_maybe_ref(next);
                        match input.next_inner() {
                                (_, Some(token)) if next.borrow_as_t() == token.borrow() => None,
                                (_, found) => Some(Error::expected_token_found_or_eof(
                                        Span::new_usize(input.span_since(befunge)),
                                        vec![next.into_clone()],
                                        found.map(MaybeRef::Val),
                                )),
                        }
                }) {
                        Err((input, err))
                } else {
                        Ok((input, seq.clone()))
                }
        }
}

pub fn tuple<I: InputType, E: ParserExtras<I>, O1, O2, A: Parser<I, O1, E>, B: Parser<I, O2, E>>(
        tuple: (A, B),
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, (O1, O2)> {
        move |input| {
                let (input, result1) = input.parse(&tuple.0)?;
                let (input, result2) = input.parse(&tuple.1)?;
                Ok((input, (result1, result2)))
        }
}

#[cfg(test)]
mod test {
        use crate::{
                input::InputOwned,
                parser::{Parser, SimpleExtras},
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
}
