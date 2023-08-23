use super::*;

#[derive(Copy, Clone)]
pub struct Repeated<P>(P);

impl<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>> Parser<I, Vec<O>, E> for Repeated<P> {
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
