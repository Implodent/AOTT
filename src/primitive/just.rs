use crate::pfn_type;

use super::*;

/// Parses a sequence of tokens `seq`.
#[inline(always)]
pub fn just<'a, I: InputType, T: OrderedSeq<'a, I::Token> + Clone, E: ParserExtras<I>>(
        seq: T,
) -> pfn_type!(I, T, E)
where
        I::Token: PartialEq + Clone + 'static,
{
        move |input| {
                if let Some(err) = seq.seq_iter().find_map(|next| {
                        let befunge = input.offset;
                        let next = T::to_maybe_ref(next);
                        match input.next_inner() {
                                (_, Some(token)) if next.borrow_as_t() == token.borrow() => None,
                                (_, found) => Some(Error::expected_token_found_or_eof(
                                        input.span_since(befunge),
                                        vec![next.into_clone()],
                                        found,
                                )),
                        }
                }) {
                        Err(err)
                } else {
                        Ok(seq.clone())
                }
        }
}
