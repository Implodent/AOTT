use super::*;

/// Parses a sequence of tokens [`seq`].
pub fn just<'a, I: InputType, T: OrderedSeq<'a, I::Token> + Clone, E: ParserExtras<I>>(
        seq: T,
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, T>
where
        I::Token: PartialEq + Clone + 'static,
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
