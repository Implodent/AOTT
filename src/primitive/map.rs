use core::marker::PhantomData;

use super::*;

pub struct Or<A, B>(pub(crate) A, pub(crate) B);

impl<A, B, I: InputType, O, E: ParserExtras<I>> Parser<I, O, E> for Or<A, B>
where
        A: Parser<I, O, E>,
        B: Parser<I, O, E>,
{
        fn explode<'parse, M: Mode>(&self, inp: Input<'parse, I, E>) -> PResult<'parse, I, E, M, O>
        where
                Self: Sized,
        {
                let befunge = inp.save();
                match self.0.explode::<M>(inp) {
                        real @ (_, Ok(_)) => real,
                        (mut inp, Err(())) => {
                                inp.rewind(befunge);
                                self.1.explode::<M>(inp)
                        }
                }
        }
        explode_extra!(O);
}

pub struct Map<A, O, F, U>(
        pub(crate) A,
        pub(crate) PhantomData<O>,
        pub(crate) F,
        pub(crate) PhantomData<U>,
);
impl<I: InputType, O, E: ParserExtras<I>, U, A: Parser<I, O, E>, F: Fn(O) -> U> Parser<I, U, E>
        for Map<A, O, F, U>
{
        fn explode<'parse, M: Mode>(&self, inp: Input<'parse, I, E>) -> PResult<'parse, I, E, M, U>
        where
                Self: Sized,
        {
                let (inp, out) = self.0.explode::<M>(inp);
                (inp, out.map(|o| M::map(o, |ou| self.2(ou))))
        }

        fn explode_emit<'parse>(&self, inp: Input<'parse, I, E>) -> PResult<'parse, I, E, Emit, U> {
                self.explode::<Emit>(inp)
        }
        fn explode_check<'parse>(
                &self,
                inp: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, Check, U> {
                self.explode::<Check>(inp)
        }
}

pub struct To<A, O, U>(pub(crate) A, pub(crate) U, pub(crate) PhantomData<O>);
impl<I: InputType, O, E: ParserExtras<I>, U: Clone, A: Parser<I, O, E>> Parser<I, U, E>
        for To<A, O, U>
{
        fn explode<'parse, M: Mode>(&self, inp: Input<'parse, I, E>) -> PResult<'parse, I, E, M, U>
        where
                Self: Sized,
        {
                let (inp, out) = self.0.explode::<M>(inp);
                (inp, out.map(|_| M::bind(|| self.1.clone())))
        }
        fn explode_emit<'parse>(&self, inp: Input<'parse, I, E>) -> PResult<'parse, I, E, Emit, U> {
                let (inp, out) = self.0.explode_emit(inp);
                (inp, out.map(|_| self.1.clone()))
        }
        fn explode_check<'parse>(
                &self,
                inp: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, Check, U> {
                self.0.explode_check(inp)
        }
}

pub struct TryMap<A, F, O, U>(
        pub(crate) A,
        pub(crate) F,
        pub(crate) PhantomData<O>,
        pub(crate) PhantomData<U>,
);
impl<
                I: InputType,
                O,
                E: ParserExtras<I>,
                U,
                F: Fn(O) -> Result<U, E::Error>,
                A: Parser<I, O, E>,
        > Parser<I, U, E> for TryMap<A, F, O, U>
{
        fn explode<'parse, M: Mode>(&self, inp: Input<'parse, I, E>) -> PResult<'parse, I, E, M, U>
        where
                Self: Sized,
        {
                let (mut inp, out) = self.0.explode_emit(inp);
                let Ok(ok) = out else {
                        return (inp, Err(()));
                };

                match self.1(ok) {
                        Ok(help) => (inp, Ok(M::bind(|| help))),
                        Err(err) => {
                                inp.errors.alt = Some(Located {
                                        pos: inp.offset,
                                        err,
                                });
                                (inp, Err(()))
                        }
                }
        }
        fn explode_emit<'parse>(&self, inp: Input<'parse, I, E>) -> PResult<'parse, I, E, Emit, U> {
                let (mut inp, out) = self.0.explode_emit(inp);
                let Ok(ok) = out else {
                        return (inp, Err(()));
                };

                match self.1(ok) {
                        Ok(help) => (inp, Ok(help)),
                        Err(err) => {
                                inp.errors.alt = Some(Located {
                                        pos: inp.offset,
                                        err,
                                });
                                (inp, Err(()))
                        }
                }
        }
        fn explode_check<'parse>(
                &self,
                inp: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, Check, U> {
                self.explode::<Check>(inp)
        }
}
