use std::marker::PhantomData;

use crate::{
        container::Seq, error::LabelError, input::SliceInput, iter::IterParser, parser::Check,
        pfn_type,
};

use super::*;

#[derive(Copy, Clone, Debug)]
pub struct Repeated<P, O> {
        pub(crate) parser: P,
        pub(crate) at_least: usize,
        // Slightly evil: should be `Option<usize>`, but we encode `!0` as 'no cap' because it's so large
        pub(crate) at_most: u64,
        pub(crate) phantom: PhantomData<O>,
}

impl<P, O> Repeated<P, O> {
        pub fn at_least(self, at_least: usize) -> Self {
                Self { at_least, ..self }
        }
        pub fn at_most(self, at_most: usize) -> Self {
                Self {
                        at_most: at_most as u64,
                        ..self
                }
        }
        pub fn exactly(self, exactly: usize) -> Self {
                Self {
                        at_least: exactly,
                        at_most: exactly as u64,
                        ..self
                }
        }
}

fn repeated_impl<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>, M: Mode>(
        this: &Repeated<P, O>,
        input: &mut Input<I, E>,
        state: &mut usize,
) -> Result<Option<M::Output<O>>, E::Error>
{
        if this.at_most != !0 && *state >= this.at_most as usize {
                return Ok(None);
        }

        let value = match M::invoke(&this.parser, input) {
                Ok(ok) => ok,
                Err(e) => {
                        if *state >= this.at_least {
                                return Ok(None);
                        } else {
                                return Err(e);
                        }
                }
        };

        *state += 1;

        Ok(Some(value))
}

impl<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>> Parser<I, (), E> for Repeated<P, O> {
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                let mut state = self.create_state(input)?;
                while let Some(_) = self.check_next(input, &mut state)? {}

                Ok(())
        }

        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                let mut state = self.create_state(input)?;
                while let Some(_) = self.check_next(input, &mut state)? {}

                Ok(())
        }
}

impl<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>> IterParser<I, E> for Repeated<P, O> {
        type Item = O;
        type State = usize;

        fn create_state(&self, _input: &mut Input<I, E>) -> Result<Self::State, E::Error> {
                Ok(0)
        }

        fn next(
                &self,
                input: &mut Input<I, E>,
                state: &mut Self::State,
        ) -> Result<Option<Self::Item>, E::Error> {
                repeated_impl::<_, _, _, _, Emit>(self, input, state)
        }

        fn check_next(
                &self,
                input: &mut Input<I, E>,
                state: &mut Self::State,
        ) -> Result<Option<()>, E::Error> {
                repeated_impl::<_, _, _, _, Check>(self, input, state)
        }
}

pub struct Slice<'a, I, E, O, P>(P, PhantomData<&'a (I, O, E)>);

impl<'a, I: InputType + SliceInput<'a>, E: ParserExtras<I>, O, P: Parser<I, O, E>>
        Parser<I, I::Slice, E> for Slice<'a, I, E, O, P>
{
        #[inline(always)]
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.0.check_with(input)
        }
        #[inline(always)]
        fn parse_with<'parse>(&self, input: &mut Input<I, E>) -> PResult<I, I::Slice, E> {
                let before = input.offset;
                self.0.check_with(input)?;
                Ok(input.input.slice(input.span_since(before)))
        }
}

pub fn slice<'a, I: InputType + SliceInput<'a>, E: ParserExtras<I>, O, P: Parser<I, O, E>>(
        parser: P,
) -> Slice<'a, I, E, O, P> {
        Slice(parser, PhantomData)
}

#[track_caller]
pub fn with_slice<
        'parse,
        'a,
        I: InputType + SliceInput<'a>,
        E: ParserExtras<I>,
        O,
        F: Fn(&mut Input<'parse, I, E>) -> PResult<I, O, E>,
>(
        input: &mut Input<'parse, I, E>,
        f: F,
) -> PResult<I, I::Slice, E> {
        let before = input.offset;
        let _ = f(input)?;
        let slice = input.input.slice(input.span_since(before));
        Ok(slice)
}

#[derive(Clone, Debug, PartialEq, Eq, derive_more::Display)]
#[display(bound = "Item: std::fmt::Debug")]
pub enum SeqLabel<Item> {
        #[display(fmt = "expected one of {_0:?}")]
        OneOf(Vec<Item>),
        #[display(fmt = "expected anything but any of {_0:?}")]
        NoneOf(Vec<Item>),
}

/// A parser that accepts only one token out of the `things`.
/// For example, you could pass a `&str` as `things`, and it would result in a parser,
/// that would match any character that `things` contains.
/// That works the same with an array, and really, anything that implements `Seq<I::Token>`.
pub fn one_of<'a, I: InputType, E: ParserExtras<I>, T: Seq<'a, I::Token>>(
        things: T,
) -> pfn_type!(I, I::Token, E)
where
        I::Token: PartialEq + Clone,
        E::Error: LabelError<I, SeqLabel<I::Token>>,
{
        move |input| {
                filter(
                        |thing| things.contains(thing),
                        SeqLabel::OneOf(things.seq_iter().map(|x| x.borrow().clone()).collect()),
                )
                .parse_with(input)
        }
}

/// A parser that accepts any token **except** ones contained in `things`.
/// ```
/// # use aott::prelude::*;
/// let parser = none_of::<&str, extra::Err<_>, _>("bcd");
/// assert_eq!(parser.parse("abcd"), Ok('a'));
/// ```
pub fn none_of<'a, I: InputType, E: ParserExtras<I>, T: Seq<'a, I::Token>>(
        things: T,
) -> pfn_type!(I, I::Token, E)
where
        I::Token: PartialEq + Clone,
        E::Error: LabelError<I, SeqLabel<I::Token>>,
{
        move |input| {
                filter(
                        |thing| !things.contains(thing),
                        SeqLabel::NoneOf(things.seq_iter().map(|x| x.borrow().clone()).collect()),
                )
                .parse_with(input)
        }
}

/// A parser that parser the content, preceded by the `start_delimiter` and terminated by the `end_delimiter`.
///
/// # Example
/// ```
/// # use aott::prelude::*;
/// let input = "\"h\"";
/// let parser = delimited(just("\""), any::<_, extra::Err<_>>, just("\""));
/// assert_eq!(parser.parse(input), Ok('h'));
/// ```
pub fn delimited<I: InputType, E: ParserExtras<I>, O, O1, O2>(
        start_delimiter: impl Parser<I, O2, E>,
        content_parser: impl Parser<I, O, E>,
        end_delimiter: impl Parser<I, O1, E>,
) -> pfn_type!(I, O, E) {
        move |input| {
                start_delimiter.check_with(input)?;
                let content = content_parser.parse_with(input)?;
                end_delimiter.check_with(input)?;

                Ok(content)
        }
}

#[derive(Copy, Clone)]
pub struct SeparatedBy<P, D, O, OD> {
        pub(crate) parser: P,
        pub(crate) delimiter: D,
        pub(crate) at_least: usize,
        // Slightly evil: should be `Option<usize>`, but we encode `!0` as 'no cap' because it's so large
        pub(crate) at_most: u64,
        pub(crate) allow_leading: bool,
        pub(crate) allow_trailing: bool,
        pub(crate) phantom: PhantomData<(O, OD)>,
}

impl<P, D, O, OD> SeparatedBy<P, D, O, OD> {
        pub fn at_least(self, at_least: usize) -> Self {
                Self { at_least, ..self }
        }
        pub fn at_most(self, at_most: usize) -> Self {
                Self {
                        at_most: at_most as u64,
                        ..self
                }
        }
        pub fn exactly(self, exactly: usize) -> Self {
                Self {
                        at_least: exactly,
                        at_most: exactly as u64,
                        ..self
                }
        }
        pub fn allow_leading(mut self) -> Self {
                self.allow_leading = true;
                self
        }
        pub fn allow_trailing(mut self) -> Self {
                self.allow_trailing = true;
                self
        }
}

fn sep_impl<
        I: InputType,
        O,
        OD,
        E: ParserExtras<I>,
        P: Parser<I, O, E>,
        D: Parser<I, OD, E>,
        M: Mode,
>(
        this: &SeparatedBy<P, D, O, OD>,
        input: &mut Input<I, E>,
        state: &mut usize,
) -> Result<Option<M::Output<O>>, E::Error>
where
        E::Error: LabelError<I, SeqLabel<I::Token>>,
{
        if this.at_most != !0 && *state >= this.at_most as usize {
                if this.allow_trailing {
                        let before_delimiter = input.save();
                        if let Err(_) = this.delimiter.check_with(input) {
                                input.rewind(before_delimiter);
                        }
                }
                return Ok(None);
        }

        if *state > 0 {
                this.delimiter.check_with(input)?;
        } else if this.allow_leading && *state == 0 {
                let before_delimiter = input.save();
                if let Err(_) = this.delimiter.check_with(input) {
                        input.rewind(before_delimiter);
                }
        }

        match M::invoke(&this.parser, input) {
                Ok(value) => {
                        *state += 1;

                        Ok(Some(value))
                }
                Err(e) => {
                        if *state >= this.at_least {
                                Ok(None)
                        } else { Err(e) }
                }
        }
}

impl<I: InputType, O, OD, E: ParserExtras<I>, P: Parser<I, O, E>, D: Parser<I, OD, E>>
        IterParser<I, E> for SeparatedBy<P, D, O, OD>
where
        E::Error: LabelError<I, SeqLabel<I::Token>>,
{
        type Item = O;
        type State = usize;

        fn create_state(&self, _input: &mut Input<I, E>) -> Result<Self::State, E::Error> {
                Ok(0)
        }

        fn next(
                &self,
                input: &mut Input<I, E>,
                state: &mut Self::State,
        ) -> Result<Option<Self::Item>, E::Error> {
                sep_impl::<_, _, _, _, _, _, Emit>(self, input, state)
        }

        fn check_next(
                &self,
                input: &mut Input<I, E>,
                state: &mut Self::State,
        ) -> Result<Option<()>, E::Error> {
                sep_impl::<_, _, _, _, _, _, Check>(self, input, state)
        }
}
