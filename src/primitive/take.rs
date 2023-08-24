use core::{mem::MaybeUninit, ops::RangeTo};

use crate::{sync::MaybeSync, MaybeUninitExt};

use super::*;

pub trait TakeAmount {
        fn range(&self) -> Range<usize>;
}

impl TakeAmount for usize {
        #[allow(clippy::range_plus_one)]
        fn range(&self) -> Range<usize> {
                // fuck you clippy
                (*self)..(self + 1)
        }
}
impl TakeAmount for Range<usize> {
        fn range(&self) -> Range<usize> {
                self.clone()
        }
}
impl TakeAmount for RangeTo<usize> {
        fn range(&self) -> Range<usize> {
                0..self.end
        }
}

pub fn take<I: InputType, E: ParserExtras<I>>(
        amount: impl TakeAmount,
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, Vec<I::Token>> {
        move |mut input| {
                let before = input.offset;
                let range = amount.range();
                let mut result = vec![];
                let mut n = 0usize;
                loop {
                        if range.end < n {
                                break;
                        }
                        match input.next_or_eof() {
                                Ok(token) => {
                                        n += 1;
                                        result.push(token);
                                }
                                Err(error) => return Err((input, error)),
                        }
                }
                if n < range.start {
                        let error = Error::expected_token_found(
                                Span::new_usize(input.span_since(before)),
                                vec![],
                                crate::Maybe::Val(unsafe {
                                        input.input.next(before).1.expect("no token??")
                                }),
                        );
                        Err((input, error))
                } else {
                        Ok((input, result))
                }
        }
}

pub struct TakeExact<const A: usize>(usize);
impl<I: InputType, E: ParserExtras<I>, const A: usize> Parser<I, [I::Token; A], E>
        for TakeExact<A>
{
        fn explode<'parse, M: Mode>(
                &self,
                mut input: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, M, [I::Token; A]> {
                let mut result = MaybeUninitExt::<I::Token>::uninit_array::<A>();
                for r in &mut result {
                        match input.next_or_eof() {
                                Ok(ok) => *r = MaybeUninit::new(ok),
                                Err(err) => {
                                        input.errors.alt = Some(Located::at(input.offset, err));
                                        return (input, Err(()));
                                }
                        }
                }
                (
                        input,
                        Ok(M::bind(|| unsafe {
                                // SAFETY: we assume init because for each element of result, we initialized it
                                MaybeUninitExt::array_assume_init(result)
                        })),
                )
        }
        fn explode_emit<'parse>(
                &self,
                input: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, Emit, [I::Token; A]> {
                self.explode::<Emit>(input)
        }
        fn explode_check<'parse>(
                &self,
                mut input: Input<'parse, I, E>,
        ) -> PResult<'parse, I, E, Check, [I::Token; A]> {
                for _ in 0..A {
                        match input.next_or_eof() {
                                Ok(_) => (),
                                Err(err) => {
                                        input.errors.alt = Some(Located::at(input.offset, err));
                                        return (input, Err(()));
                                }
                        }
                }
                (input, Ok(()))
        }
}

#[must_use]
pub fn take_exact<const A: usize>() -> TakeExact<A> {
        TakeExact(A)
}
