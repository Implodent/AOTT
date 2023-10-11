use core::marker::PhantomData;

use crate::{
        error::{Error, Filtering, FundamentalError, LabelError},
        input::{Input, InputType},
        parser::*,
        pfn_type, PResult,
};

fn filter_impl<
        I: InputType,
        O,
        E: ParserExtras<I>,
        A: Parser<I, O, E>,
        F: Fn(&O) -> bool,
        M: Mode,
        L: Clone,
>(
        _mode: &M,
        this: &FilterParser<A, F, O, L>,
        input: &mut Input<I, E>,
) -> PResult<I, M::Output<O>, E>
where
        E::Error: LabelError<I, L>,
{
        let offset = input.offset;
        this.0.parse_with(input).and_then(|thing| {
                if this.1(&thing) {
                        Ok(M::bind(|| thing))
                } else {
                        let err = LabelError::from_label(
                                input.span_since(offset),
                                this.2.clone(),
                                input.current(),
                        );
                        Err(err)
                }
        })
}

pub struct FilterParser<A, F, O, L>(
        pub(crate) A,
        pub(crate) F,
        pub(crate) L,
        pub(crate) PhantomData<O>,
);

impl<I: InputType, O, E: ParserExtras<I>, A: Parser<I, O, E>, F: Fn(&O) -> bool, L: Clone>
        Parser<I, O, E> for FilterParser<A, F, O, L>
where
        E::Error: LabelError<I, L>,
{
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                filter_impl(&Check, self, input)
        }

        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O, E> {
                filter_impl(&Emit, self, input)
        }
}

#[track_caller]
pub fn filter<I: InputType, E: ParserExtras<I>, L: Clone>(
        filter: impl Fn(&I::Token) -> bool,
        label: L,
) -> impl Fn(&mut Input<I, E>) -> PResult<I, I::Token, E>
where
        E::Error: LabelError<I, L>,
{
        move |input| {
                let befunge = input.offset;
                match input.next_or_none() {
                        Some(el) if filter(&el) => Ok(el),
                        other => Err(LabelError::from_label(
                                input.span_since(befunge),
                                label.clone(),
                                other,
                        )),
                }
        }
}

pub fn filter_map<I: InputType, E: ParserExtras<I>, U, L: Clone>(
        mapper: impl Fn(I::Token) -> Option<U>,
        label: L,
) -> pfn_type!(I, U, E)
where
        I::Token: Clone,
        E::Error: LabelError<I, L>,
{
        #[cfg_attr(feature = "nightly", track_caller)]
        move |input| {
                let befunge = input.offset;
                let next = input.next()?;

                mapper(next).ok_or_else(|| {
                        LabelError::from_label(
                                input.span_since(befunge),
                                label.clone(),
                                input.current(),
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

/// Creates a [`Filtering`]
#[must_use]
pub fn filtering(what: impl Into<Cow<'static, str>>) -> Filtering {
        Filtering(what.into())
}

/// Implement `LabelError<I, Filtering>` to use `filter*` with your error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filtering(Cow<'static, str>);
