use crate::{
        error::{Error, Span},
        input::{Input, InputType},
        parser::ParserExtras,
        IResult, Maybe,
};

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
