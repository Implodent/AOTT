use core::marker::PhantomData;

use crate::{
        error::Error,
        input::{Input, InputType},
        parser::*,
        pfn_type, PResult,
};

#[track_caller]
fn filter_impl<
        I: InputType,
        O,
        E: ParserExtras<I>,
        A: Parser<I, O, E>,
        F: Fn(&O) -> bool,
        M: Mode,
>(
        _mode: &M,
        this: &FilterParser<A, F, O>,
        input: &mut Input<I, E>,
) -> PResult<I, M::Output<O>, E> {
        let offset = input.offset;
        this.0.parse_with(input).and_then(|thing| {
                if this.1(&thing) {
                        Ok(M::bind(|| thing))
                } else {
                        let err = Error::filter_failed(
                                input.span_since(offset),
                                core::panic::Location::caller(),
                                input.current().expect("what"),
                        );
                        Err(err)
                }
        })
}

pub struct FilterParser<A, F, O>(pub(crate) A, pub(crate) F, pub(crate) PhantomData<O>);
impl<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>, F: Fn(&O) -> bool> Parser<I, O, E>
        for FilterParser<A, F, O>
{
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                filter_impl(&Check, self, input)
        }

        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O, E> {
                filter_impl(&Emit, self, input)
        }
}

#[track_caller]
pub fn filter<I: InputType, E: ParserExtras<I>>(
        filter: impl Fn(&I::Token) -> bool,
) -> impl Fn(&mut Input<I, E>) -> PResult<I, I::Token, E> {
        #[cfg_attr(feature = "nightly", track_caller)]
        move |input| {
                let befunge = input.offset;
                match input.next_or_none() {
                        Some(el) if filter(&el) => Ok(el),
                        Some(other) => Err(Error::filter_failed(
                                input.span_since(befunge),
                                core::panic::Location::caller(),
                                other,
                        )),
                        None => Err(Error::unexpected_eof(input.span_since(befunge), None)),
                }
        }
}

#[track_caller]
pub fn filter_map<I: InputType, E: ParserExtras<I>, U>(
        mapper: impl Fn(I::Token) -> Option<U>,
) -> pfn_type!(I, U, E)
where
        I::Token: Clone,
{
        #[cfg_attr(feature = "nightly", track_caller)]
        move |input| {
                let befunge = input.offset;
                let next = input.next()?;

                mapper(next).ok_or_else(|| {
                        Error::filter_failed(
                                input.span_since(befunge),
                                core::panic::Location::caller(),
                                input.current().expect("eof"),
                        )
                })
        }
}

#[macro_export]
macro_rules! select {
    ($($pat:pat$(if $guard:expr)? => $res:expr),*$(,)?) => {
        $crate::primitive::filter_map(|__token| match __token {
            $($pat$(if $guard)? => Some($res),)*
            _ => None
        })
    };
}

pub struct Rewind<A>(A);
impl<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>> Parser<I, O, E> for Rewind<A> {
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                let befunge = input.save();
                let output = self.0.check_with(input)?;
                input.rewind(befunge);
                Ok(output)
        }
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O, E> {
                let befunge = input.save();
                let output = self.0.parse_with(input)?;
                input.rewind(befunge);
                Ok(output)
        }
}

/// Transforms a parser, so that when it completes, the input is rewound to where it was before parsing.
#[must_use]
pub fn rewind<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>>(parser: A) -> Rewind<A> {
        Rewind(parser)
}
