use core::marker::PhantomData;

use crate::input::SliceInput;

use super::*;

#[derive(Copy, Clone)]
pub struct Repeated<P, O, V: FromIterator<O>>(
        pub(crate) P,
        pub(crate) PhantomData<O>,
        pub(crate) PhantomData<V>,
);

impl<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>, V: FromIterator<O>>
        Parser<I, Vec<O>, E> for Repeated<P, O, V>
{
        fn explode<'parse, M: Mode>(
                &self,
                mut inp: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, M, Vec<O>>
        where
                Self: Sized,
        {
                let mut result = vec![];
                loop {
                        let befunge = inp.save();

                        match self.0.explode_emit(inp) {
                                (input, Ok(ok)) => {
                                        inp = input;
                                        result.push(ok);
                                }
                                (mut input, Err(())) => {
                                        input.rewind(befunge);
                                        input.errors.alt = None;
                                        return (input, Ok(M::bind(|| result)));
                                }
                        }
                }
        }
        fn explode_check<'parse>(
                &self,
                mut inp: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, Check, Vec<O>> {
                loop {
                        let befunge = inp.save();
                        match self.0.explode_check(inp) {
                                (input, Ok(())) => inp = input,
                                (mut input, Err(())) => {
                                        input.rewind(befunge);
                                        return (input, Ok(()));
                                }
                        }
                }
        }
        fn explode_emit<'parse>(
                &self,
                inp: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, Emit, Vec<O>> {
                self.explode::<Emit>(inp)
        }
}

pub struct Slice<'a, I, E, O, P>(P, PhantomData<&'a (I, O, E)>);

impl<'a, I: InputType + SliceInput<'a>, E: ParserExtras<I>, O, P: Parser<I, O, E>>
        Parser<I, I::Slice, E> for Slice<'a, I, E, O, P>
{
        fn explode<'parse, M: Mode>(
                &self,
                inp: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, M, I::Slice>
        where
                Self: Sized,
        {
                let start = inp.offset;
                let (inp, out) = self.0.explode_check(inp);
                if let Err(()) = out {
                        return (inp, Err(()));
                }

                let slice = inp.input.slice(inp.span_since(start));
                (inp, Ok(M::bind(|| slice)))
        }

        explode_extra!(I::Slice);
}

pub fn slice<'a, I: InputType + SliceInput<'a>, E: ParserExtras<I>, O, P: Parser<I, O, E>>(
        parser: P,
) -> Slice<'a, I, E, O, P> {
        Slice(parser, PhantomData)
}
