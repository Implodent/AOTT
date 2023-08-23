use core::marker::PhantomData;

use crate::{
        error::{Error, Located, Span},
        explode_extra,
        input::{Input, InputType},
        parser::*,
        IResult, Maybe,
};

pub struct FilterParser<A, F, O>(pub(crate) A, pub(crate) F, pub(crate) PhantomData<O>);
impl<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>, F: Fn(&O) -> bool> Parser<I, O, E>
        for FilterParser<A, F, O>
{
        fn explode<'parse, M: crate::parser::Mode>(
                &self,
                inp: Input<'parse, I, E>,
        ) -> crate::parser::PResult<'parse, I, E, M, O>
        where
                Self: Sized,
        {
                let start = inp.offset;
                let (mut inp, out) = self.0.explode_emit(inp);
                let Ok(out) = out else {
                        return (inp, Err(()));
                };

                if self.1(&out) {
                        (inp, Ok(M::bind(|| out)))
                } else {
                        let curr = inp.offset;
                        inp.offset = start;
                        inp.errors.alt = Some(Located::at(
                                inp.offset,
                                Error::expected_token_found(
                                        Span::new_usize(start..curr),
                                        vec![],
                                        Maybe::Val(inp.peek().expect("no eof error but now eof")),
                                ),
                        ));
                        inp.offset = curr;

                        (inp, Err(()))
                }
        }

        explode_extra!(O);
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
                                        Maybe::Val(other),
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
                                Maybe::Val(n),
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
