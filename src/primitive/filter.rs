use core::marker::PhantomData;

use crate::{
        error::{Error, Span},
        input::{Input, InputType},
        parser::*,
        IResult, MaybeDeref,
};

fn filter_impl<
        'parse,
        I: InputType,
        O,
        E: ParserExtras<I>,
        A: Parser<I, O, E>,
        F: Fn(&O) -> bool,
        M: Mode,
>(
        _mode: &M,
        this: &FilterParser<A, F, O>,
        input: Input<'parse, I, E>,
) -> IResult<'parse, I, E, M::Output<O>> {
        let offset = input.offset;
        this.0.parse(input).and_then(|(input, thing)| {
                if this.1(&thing) {
                        Ok((input, M::bind(|| thing)))
                } else {
                        let err = Error::expected_token_found(
                                Span::new_usize(input.span_since(offset)),
                                vec![],
                                MaybeDeref::Val(
                                        input.peek().expect("no eof error but now eof. bruh."),
                                ),
                        );
                        Err((input, err))
                }
        })
}

pub struct FilterParser<A, F, O>(pub(crate) A, pub(crate) F, pub(crate) PhantomData<O>);
impl<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>, F: Fn(&O) -> bool> Parser<I, O, E>
        for FilterParser<A, F, O>
{
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                filter_impl(&Check, self, input)
        }

        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, O> {
                filter_impl(&Emit, self, input)
        }
}

pub fn filter<I: InputType, E: ParserExtras<I>>(
        filter: impl Fn(&I::Token) -> bool,
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, I::Token> {
        move |mut input| {
                let befunge = input.offset;
                match input.next() {
                        Some(el) if filter(&el) => Ok((input, el)),
                        Some(other) => {
                                let err = Error::expected_token_found(
                                        Span::new_usize(input.span_since(befunge)),
                                        vec![],
                                        MaybeDeref::Val(other),
                                );
                                Err((input, err))
                        }
                        None => {
                                let err = Error::unexpected_eof(
                                        Span::new_usize(input.span_since(befunge)),
                                        None,
                                );
                                Err((input, err))
                        }
                }
        }
}

pub fn filter_map<I: InputType, E: ParserExtras<I>, U>(
        mapper: impl Fn(I::Token) -> Option<U>,
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, U>
where
        I::Token: Clone,
{
        move |mut input| {
                let befunge = input.offset;
                let Some(next) = input.next() else {
                        let err = Error::unexpected_eof(
                                Span::new_usize(input.span_since(befunge)),
                                None,
                        );
                        return Err((input, err));
                };

                let n = next.clone();
                if let Some(fin) = mapper(next) {
                        Ok((input, fin))
                } else {
                        let err = Error::expected_token_found(
                                Span::new_usize(input.span_since(befunge)),
                                vec![],
                                MaybeDeref::Val(n),
                        );
                        Err((input, err))
                }
        }
}

#[macro_export]
macro_rules! select {
    ($($pat:pat$(if $guard:expr)? => $res:expr)*) => {
        $crate::primitive::filter_map(|__token| match __token {
            $($pat$(if $guard)? => Some($res),)*
            _ => None
        })
    };
}

pub struct Rewind<A>(A);
impl<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>> Parser<I, O, E> for Rewind<A> {
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                let befunge = input.save();
                let (mut input, output) = self.0.check(input)?;
                input.rewind(befunge);
                Ok((input, output))
        }
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, O> {
                let befunge = input.save();
                let (mut input, output) = self.0.parse(input)?;
                input.rewind(befunge);
                Ok((input, output))
        }
}

/// Transforms a parser, so that when it completes, the input is rewound to where it was before parsing.
#[must_use]
pub fn rewind<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>>(parser: A) -> Rewind<A> {
        Rewind(parser)
}
