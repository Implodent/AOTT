use core::marker::PhantomData;

use crate::{container::Seq, input::SliceInput, parser::Check, pfn_type};

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
        I: InputType,
        O,
        E: ParserExtras<I>,
        P: Parser<I, O, E>,
        V: FromIterator<O>,
        M: Mode,
>(
        input: &mut Input<I, E>,
        this: &Repeated<P, O, V>,
        _m: &M,
) -> PResult<I, M::Output<V>, E> {
        let mut result = vec![];
        let mut count = 0usize;
        let res = loop {
                if this.at_most != !0 && count as u64 >= this.at_most {
                        break Ok(M::bind(|| result.into_iter().collect()));
                }

                let before = input.save();
                match M::invoke(&this.parser, input) {
                        Ok(o) => {
                                M::invoke_unbind(|val| result.push(val), o);
                        }
                        Err(e) => {
                                if count < this.at_least {
                                        break Err(e);
                                }
                                input.rewind(before);
                                break Ok(M::bind(|| result.into_iter().collect()));
                        }
                }

                count += 1; // what the fuck
        };
        if count < this.at_least {
                Err(Error::expected_token_found(
                        Span::new_usize(input.span_since(I::prev(input.offset))),
                        vec![],
                        crate::MaybeDeref::Val(input.current().expect("huh")),
                ))
        } else {
                res
        }
}

impl<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>, V: FromIterator<O>> Parser<I, V, E>
        for Repeated<P, O, V>
{
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                repeated_impl(input, self, &Check)
        }
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, V, E> {
                repeated_impl(input, self, &Emit)
        }
}

pub struct Slice<'a, I, E, O, P>(P, PhantomData<&'a (I, O, E)>);

impl<'a, I: InputType + SliceInput<'a>, E: ParserExtras<I>, O, P: Parser<I, O, E>>
        Parser<I, I::Slice, E> for Slice<'a, I, E, O, P>
{
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.0.check_with(input)
        }
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

/// A parser that accepts only one token out of the `things`.
/// For example, you could pass a `&str` as `things`, and it would result in a parser,
/// that would match any character that `things` contains.
/// That works the same with an array, and really, anything that implements `Seq<I::Token>`.
pub fn one_of<'a, I: InputType, E: ParserExtras<I>, T: Seq<'a, I::Token>>(
        things: T,
) -> pfn_type!(I, I::Token, E)
where
        I::Token: PartialEq,
{
        move |input| filter(|thing| things.contains(thing)).parse_with(input)
}

/// A parser that accepts any token **except** ones contained in `things`.
/// ```
/// # use aott::prelude::*;
/// let input = "abcd";
/// assert_eq!(none_of::<&str, extra::Err<_>, _>("bcd").parse_from(&input).into_result(), Ok('a'));
/// ```
pub fn none_of<'a, I: InputType, E: ParserExtras<I>, T: Seq<'a, I::Token>>(
        things: T,
) -> pfn_type!(I, I::Token, E)
where
        I::Token: PartialEq,
{
        move |input| filter(|thing| !things.contains(thing)).parse_with(input)
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
) -> pfn_type!(I, O, E) {
        move |input| {
                start_delimiter.check_with(input)?;
                let content = content_parser.parse_with(input)?;
                end_delimiter.check_with(input)?;

                Ok(content)
        }
}
