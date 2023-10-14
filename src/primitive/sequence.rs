use core::marker::PhantomData;

use crate::{
        container::Seq,
        error::{BuiltinLabel, LabelError},
        input::SliceInput,
        iter::IterParser,
        parser::Check,
        pfn_type, PResult,
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
) -> Result<Option<M::Output<O>>, E::Error> {
        if this.at_most != !0 && *state >= this.at_most as usize {
                return Ok(None);
        }

        let before = input.offset;

        let Some(value) = M::invoke(&this.parser, input).ok() else {
                return Ok(None);
        };

        *state += 1;

        if *state < this.at_least {
                return Err(LabelError::from_label(
                        input.span_since(before),
                        BuiltinLabel::NotEnoughElements {
                                expected_amount: this.at_least,
                                found_amount: *state,
                        },
                        input.current(),
                ));
        }

        Ok(Some(value))
}

impl<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>> Parser<I, Vec<O>, E>
        for Repeated<P, O>
{
        fn parse_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
                let mut state = self.create_state(input)?;
                while let Some(_) = self.next(input, &mut state)? {}

                Ok(())
        }

        fn check_with(&self, input: &mut Input<I, E>) -> $1PResult<$2, E> {
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

struct Slice<'a, I, E, O, P>(P, PhantomData<&'a (I, O, E)>);

impl<'a, I: SliceInput + 'a, E: ParserExtras<I>, O, P: Parser<I, O, E>> Parser<I, &'a I::Slice, E>
        for Slice<'a, I, E, O, P>
{
        #[inline(always)]
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<&'a I::Slice, E> {
                self.0.check_with(input)
        }

        #[inline(always)]
        fn parse_with<'parse>(&self, input: &mut Input<I, E>) -> PResult<&'a I::Slice, E> {
                let before = input.offset;
                self.0.check_with(input)?;
                Ok(input.input.slice(input.span_since(before)))
        }
}

pub fn slice<'a, I: SliceInput + 'a, E: ParserExtras<I>, O, P: Parser<I, O, E>>(
        parser: P,
) -> Slice<'a, I, E, O, P> {
        Slice(parser, PhantomData)
}

pub fn with_slice<
        'parse,
        I: SliceInput,
        E: ParserExtras<I>,
        O,
        F: Fn(&mut Input<'parse, I, E>) -> PResult<O, E>,
>(
        input: &mut Input<'parse, I, E>,
        f: F,
) -> PResult<&'parse I::Slice, E> {
        let before = input.offset;
        let _ = f(input)?;
        let slice = input.input.slice(input.span_since(before));
        Ok(slice)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeqLabel<Item> {
        OneOf(Vec<Item>),
        NoneOf(Vec<Item>),
}

/// A parser that accepts only one token out of the `things`.
/// For example, you could pass a `&str` as `things`, and it would result in a parser,
/// that would match any character that `things` contains.
/// That works the same with an array, and really, anything that implements `Seq<I::Token>`.
pub fn one_of<'a, I: InputType + 'a, E: ParserExtras<I> + 'a, T: Seq<'a, I::Token> + 'a>(
        things: T,
) -> impl Parser<I, I::Token, E> + 'a
where
        I::Token: PartialEq + Clone,
        E::Error: LabelError<I, SeqLabel<I::Token>>,
{
        filter(
                |thing| things.contains(thing),
                SeqLabel::OneOf(things.seq_iter().map(|x| x.borrow().clone()).collect()),
        )
}

/// A parser that accepts any token **except** ones contained in `things`.
/// ```
/// # use aott::prelude::*;
/// let parser = none_of::<&str, extra::Err<_>, _>("bcd");
/// assert_eq!(parser.parse("abcd"), Ok('a'));
/// ```
pub fn none_of<'a, I: InputType + 'a, E: ParserExtras<I> + 'a, T: Seq<'a, I::Token> + 'a>(
        things: T,
) -> impl Parser<I, I::Token, E> + 'a
where
        I::Token: PartialEq + Clone,
        E::Error: LabelError<I, SeqLabel<I::Token>>,
{
        filter(
                |thing| !things.contains(thing),
                SeqLabel::NoneOf(things.seq_iter().map(|x| x.borrow().clone()).collect()),
        )
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
) -> impl Parser<I, O, E> {
        start_delimiter
                .ignore_then(content_parser)
                .then_ignore(end_delimiter)
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

        let before = input.offset;

        if *state > 0 {
                this.delimiter.check_with(input)?;
        } else if this.allow_leading && *state == 0 {
                let before_delimiter = input.save();
                if let Err(_) = this.delimiter.check_with(input) {
                        input.rewind(before_delimiter);
                }
        }

        let Some(value) = M::invoke(&this.parser, input).ok() else {
                return Ok(None);
        };

        *state += 1;

        if *state < this.at_least {
                return Err(LabelError::from_label(
                        input.span_since(before),
                        BuiltinLabel::NotEnoughElements {
                                expected_amount: this.at_least,
                                found_amount: *state,
                        },
                        input.current(),
                ));
        }

        Ok(Some(value))
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
