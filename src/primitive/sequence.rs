use core::marker::PhantomData;

use crate::{container::Seq, input::SliceInput, parser::Check};

use super::*;

#[derive(Copy, Clone)]
pub struct Repeated<P, O, V: FromIterator<O> = Vec<O>> {
        pub(crate) parser: P,
        pub(crate) phantom: PhantomData<(O, V)>,
        pub(crate) at_least: usize,
        // Slightly evil: should be `Option<usize>`, but we encode `!0` as 'no cap' because it's so large
        pub(crate) at_most: u64,
}

impl<P, O, V: FromIterator<O>> Repeated<P, O, V> {
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
fn repeated_impl<
        'parse,
        I: InputType,
        O,
        E: ParserExtras<I>,
        P: Parser<I, O, E>,
        V: FromIterator<O>,
        M: Mode,
>(
        mut input: Input<'parse, I, E>,
        this: &Repeated<P, O, V>,
        _m: &M,
) -> IResult<'parse, I, E, M::Output<V>> {
        let mut result = vec![];
        let mut count = 0usize;
        let res = loop {
                if this.at_most != !0 && count as u64 >= this.at_most {
                        break Ok((input, M::bind(|| result.into_iter().collect())));
                }

                let before = input.save();
                match M::invoke(&this.parser, input) {
                        Ok((inp, o)) => {
                                input = inp;
                                M::invoke_unbind(|val| result.push(val), o);
                        }
                        Err((mut inp, e)) => {
                                if count < this.at_least {
                                        break Err((inp, e));
                                }
                                inp.rewind(before);
                                break Ok((inp, M::bind(|| result.into_iter().collect())));
                        }
                }

                count += 1; // what the fuck
        };
        if count < this.at_least {
                let inp = res?.0;
                let err = Error::expected_token_found(
                        Span::new_usize(inp.span_since(I::prev(inp.offset))),
                        vec![],
                        crate::MaybeDeref::Val(inp.peek().expect("huh")),
                );
                Err((inp, err))
        } else {
                res
        }
}

impl<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>, V: FromIterator<O>> Parser<I, V, E>
        for Repeated<P, O, V>
{
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                repeated_impl(input, self, &Check)
        }

        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, V> {
                repeated_impl(input, self, &Emit)
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

pub fn with_slice<
        'parse,
        'a,
        I: InputType + SliceInput<'a>,
        E: ParserExtras<I>,
        O,
        F: Fn(Input<'parse, I, E>) -> IResult<'parse, I, E, O>,
>(
        input: Input<'parse, I, E>,
        f: F,
) -> IResult<'parse, I, E, I::Slice> {
        let before = input.offset;
        let (input, _) = f(input)?;
        let slice = input.input.slice(input.span_since(before));
        Ok((input, slice))
}

/// A parser that accepts only one token out of the `things`.
/// For example, you could pass a `&str` as `things`, and it would result in a parser,
/// that would match any character that `things` contains.
/// That works the same with an array, and really, anything that implements `Seq<I::Token>`.
pub fn one_of<'a, I: InputType, E: ParserExtras<I>, T: Seq<'a, I::Token>>(
        things: T,
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, I::Token>
where
        I::Token: PartialEq,
{
        move |input| any.filter(|thing| things.contains(thing)).parse(input)
}

/// A parser that accepts any token **except** ones contained in `things`.
/// ```
/// # use aott::prelude::*;
/// let input = "abcd";
/// assert_eq!(none_of::<&str, extra::Err<_>, _>("bcd").parse_from(&input).into_result(), Ok('a'));
/// ```
pub fn none_of<'a, I: InputType, E: ParserExtras<I>, T: Seq<'a, I::Token>>(
        things: T,
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, I::Token>
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
/// let parser = delimited(just("\""), any::<_, extra::Err<_>>, just("\""));
/// assert_eq!(parser.parse_from(&input).into_result(), Ok('h'));
/// ```
pub fn delimited<I: InputType, E: ParserExtras<I>, O, O1, O2>(
        start_delimiter: impl Parser<I, O2, E>,
        content_parser: impl Parser<I, O, E>,
        end_delimiter: impl Parser<I, O1, E>,
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, O> {
        move |input| {
                let (input, _) = start_delimiter.parse(input)?;
                let (input, content) = content_parser.parse(input)?;
                let (input, _) = end_delimiter.parse(input)?;

                Ok((input, content))
        }
}
