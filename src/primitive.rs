// just, filter, end, nothing, one_of, none_of, separated_by, filter_map,
// select!, take, take_while, take_while_bounded

use crate::{
        container::OrderedSeq,
        error::Error,
        input::{Input, InputType},
        parser::ParserExtras,
        IResult,
};

pub fn just<'a, I: InputType, T: OrderedSeq<'a, I::Token>, E: ParserExtras<I>>(
        seq: T,
) -> impl Fn(Input<I, E>) -> IResult<I, I::Token, E> {
        move |input| {}
}
