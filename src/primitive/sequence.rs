use core::marker::PhantomData;

use crate::{container::Seq, input::SliceInput};

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

/// A parser that accepts only one token out of the `things`.
/// For example, you could pass a `&str` as `things`, and it would result in a parser,
/// that would match any character that `things` contains.
/// That works the same with an array, and really, anything that implements `Seq<I::Token>`.
pub fn one_of<'parse, 'a, I: InputType, E: ParserExtras<I>, T: Seq<'a, I::Token>>(
        things: T,
) -> impl Fn(Input<'parse, I, E>) -> IResult<'parse, I, E, I::Token>
where
        I::Token: PartialEq,
{
        move |input| any.filter(|thing| things.contains(thing)).parse(input)
}

/// A parser that accepts any token **except** ones contained in `things`.
/// ```
/// # use aott::prelude::*;
/// let input = "abcd";
/// let (_, value) = none_of("bcd")(Input::<&str, SimpleExtras<&str>>::new(&input)).unwrap();
/// assert_eq!(value, 'a');
/// ```
pub fn none_of<'parse, 'a, I: InputType, E: ParserExtras<I>, T: Seq<'a, I::Token>>(
        things: T,
) -> impl Fn(Input<'parse, I, E>) -> IResult<'parse, I, E, I::Token>
where
        I::Token: PartialEq,
{
        move |input| any.filter(|thing| !things.contains(thing)).parse(input)
}

/// A parser that parser the content, preceded by the `start_delimiter` and terminated by the `end_delimiter`.
///
/// # Example
/// ```
/// # use aott::prelude::*;
/// let input = "\"h\"";
/// let parser = delimited(just("\""), any::<_, SimpleExtras<_>>, just("\""));
/// let (_, value) = parser(Input::new(&input)).unwrap();
/// assert_eq!(value, 'h');
/// ```
pub fn delimited<'parse, I: InputType, E: ParserExtras<I>, O, O1, O2>(
        start_delimiter: impl Parser<I, O2, E>,
        content_parser: impl Parser<I, O, E>,
        end_delimiter: impl Parser<I, O1, E>,
) -> impl Fn(Input<'parse, I, E>) -> IResult<'parse, I, E, O> {
        move |input| {
                let (input, _) = start_delimiter.parse(input)?;
                let (input, content) = content_parser.parse(input)?;
                let (input, _) = end_delimiter.parse(input)?;

                Ok((input, content))
        }
}
