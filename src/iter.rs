use core::marker::PhantomData;

use crate::{
        input::{Input, InputType},
        parser::{Check, Emit, Mode, ParserExtras},
        prelude::Parser,
};

pub trait IterParser<I: InputType, E: ParserExtras<I>> {
        type Item;
        type State;

        fn next(
                &self,
                input: &mut Input<I, E>,
                state: &mut Self::State,
        ) -> Result<Option<Self::Item>, E::Error>;

        fn check_next(
                &self,
                input: &mut Input<I, E>,
                state: &mut Self::State,
        ) -> Result<Option<()>, E::Error>;

        /// Creates the state that the parser would use.
        fn create_state(&self, input: &mut Input<I, E>) -> Result<Self::State, E::Error>;

        fn collect<B: FromIterator<Self::Item>>(self) -> Collect<Self, B>
        where
                Self: Sized,
        {
                Collect(self, PhantomData)
        }
}

pub struct Collect<P, B>(P, PhantomData<B>);

impl<I: InputType, P: IterParser<I, E>, B: FromIterator<P::Item>, E: ParserExtras<I>>
        Parser<I, B, E> for Collect<P, B>
{
        fn parse_with(&self, input: &mut Input<I, E>) -> crate::PResult<I, B, E> {
                Ok(IterParse {
                        parser: &self.0,
                        input,
                        state: None,
                        mode: PhantomData::<Emit>,
                }
                .collect())
        }

        fn check_with(&self, input: &mut Input<I, E>) -> crate::PResult<I, (), E> {
                Ok(IterParse {
                        parser: &self.0,
                        input,
                        state: None,
                        mode: PhantomData::<Check>,
                }
                .collect())
        }
}

#[doc(hidden)]
pub struct IterParse<
        'a,
        'input,
        'parse,
        I: InputType,
        E: ParserExtras<I>,
        P: IterParser<I, E>,
        M: Mode,
> {
        pub parser: &'a P,
        pub input: &'input mut Input<'parse, I, E>,
        pub state: Option<P::State>,
        pub mode: PhantomData<M>,
}

impl<'a, 'input, 'parse, I: InputType, E: ParserExtras<I>, P: IterParser<I, E>> Iterator
        for IterParse<'a, 'input, 'parse, I, E, P, Emit>
{
        type Item = P::Item;

        fn next(&mut self) -> Option<Self::Item> {
                let state = if let Some(state) = self.state.as_mut() {
                        state
                } else {
                        self.state
                                .insert(self.parser.create_state(self.input).ok()?)
                };

                self.parser.next(self.input, state).ok().flatten()
        }
}

impl<'a, 'input, 'parse, I: InputType, E: ParserExtras<I>, P: IterParser<I, E>> Iterator
        for IterParse<'a, 'input, 'parse, I, E, P, Check>
{
        type Item = ();

        fn next(&mut self) -> Option<Self::Item> {
                let state = if let Some(state) = self.state.as_mut() {
                        state
                } else {
                        self.state
                                .insert(self.parser.create_state(self.input).ok()?)
                };

                self.parser.check_next(self.input, state).ok().flatten()
        }
}
