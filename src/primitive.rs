// just, filter, end, nothing, one_of, none_of, separated_by, filter_map,
// select!, take, take_while, take_while_bounded

use crate::{container::OrderedSeq, input::InputType, parser::ParserExtras};

pub fn just<'a, I: InputType, T: OrderedSeq<'a, I::Token>, E: ParserExtras<I>>(seq: T) {
        todo!()
}
