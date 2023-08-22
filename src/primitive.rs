// just, filter, end, nothing, one_of, none_of, separated_by, filter_map,
// select!, take, take_while, take_while_bounded

use core::borrow::Borrow;

use crate::{
        container::OrderedSeq,
        error::{Error, Span},
        input::{Input, InputType},
        parser::ParserExtras,
        IResult, MaybeRef,
};

pub fn just<'a, I: InputType, T: OrderedSeq<'a, I::Token>, E: ParserExtras<I>>(
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
                        (input, Err(err))
                } else {
                        (input, Ok(seq))
                }
        }
}

#[cfg(test)]
mod tests {
        use crate::{
                input::InputOwned,
                parser::{Parser, SimpleExtras},
        };

        use super::just;

        #[test]
        fn just_parses_a() {
                let mut input =
                        InputOwned::<&'static str, SimpleExtras<&'static str>>::from_input("abc");
                let inp = input.as_ref_at_zero();
                let parser = just('a');

                assert_eq!(parser.parse_from_input(inp).output, Some('a'));
        }
}
