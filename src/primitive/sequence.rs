use core::marker::PhantomData;

use crate::input::SliceInput;

use super::*;

#[derive(Copy, Clone)]
pub struct Repeated<P, O, V: FromIterator<O>> {
        pub(crate) parser: P,
        pub(crate) phantom: PhantomData<(O, V)>,
}

impl<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>, V: FromIterator<O>>
        Parser<I, Vec<O>, E> for Repeated<P, O, V>
{
        fn check<'parse>(&self, mut input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                loop {
                        let before = input.save();
                        match self.parser.check(input) {
                                Ok((inp, ())) => input = inp,
                                Err((mut inp, _)) => {
                                        inp.rewind(before);
                                        break Ok((inp, ()));
                                }
                        }
                }
        }
        fn parse<'parse>(&self, mut input: Input<'parse, I, E>) -> IResult<'parse, I, E, Vec<O>> {
                let mut result = vec![];
                loop {
                        let before = input.save();
                        match self.parser.parse(input) {
                                Ok((inp, thimg)) => {
                                        input = inp;
                                        result.push(thimg);
                                }
                                Err((mut inp, _)) => {
                                        inp.rewind(before);
                                        break Ok((inp, result));
                                }
                        }
                }
        }
}

pub struct Slice<'a, I, E, O, P>(P, PhantomData<&'a (I, O, E)>);

impl<'a, I: InputType + SliceInput<'a>, E: ParserExtras<I>, O, P: Parser<I, O, E>>
        Parser<I, I::Slice, E> for Slice<'a, I, E, O, P>
{
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.0.check(input)
        }
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, I::Slice> {
                let before = input.offset;
                let (input, ()) = self.0.check(input)?;
                let slice = input.input.slice(input.span_since(before));

                Ok((input, slice))
        }
}

pub fn slice<'a, I: InputType + SliceInput<'a>, E: ParserExtras<I>, O, P: Parser<I, O, E>>(
        parser: P,
) -> Slice<'a, I, E, O, P> {
        Slice(parser, PhantomData)
}
