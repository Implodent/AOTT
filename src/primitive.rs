// just, filter, end, nothing, one_of, none_of, separated_by, filter_map,
// select!, take, take_while, take_while_bounded

use crate::{container::OrderedSeq, input::{InputOf, Input}, parser::ParserExtras, IResult};

pub fn just<'a, I: Input, T: OrderedSeq<'a, I::Token>, E: ParserExtras<I>>(
        seq: T,
) -> impl Fn(InputOf<I>) -> IResult<I, I::Token, E> {
    move |input| {
        seq.seq_iter().find_map(|next| {
            match input.next(offset)
        })
    }
}
