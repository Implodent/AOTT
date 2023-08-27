use core::{mem::MaybeUninit, ops::RangeTo};

use crate::MaybeUninitExt;

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
        fn parse<'parse>(
                &self,
                mut inp: Input<'parse, I, E>,
        ) -> IResult<'parse, I, E, [I::Token; A]> {
                let mut result: [MaybeUninit<I::Token>; A] = MaybeUninitExt::uninit_array();

                for i in 0..A {
                        result[i] = MaybeUninit::new(match inp.next_or_eof() {
                                Ok(ok) => ok,
                                Err(e) => return Err((inp, e)),
                        });
                }

                Ok((inp, unsafe { MaybeUninitExt::array_assume_init(result) }))
        }
        fn check<'parse>(&self, mut inp: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                for _ in 0..A {
                        if let Err(e) = inp.next_or_eof() {
                                return Err((inp, e));
                        }
                }

                Ok((inp, ()))
        }
}

#[must_use]
pub fn take_exact<const A: usize>() -> TakeExact<A> {
        TakeExact(A)
}
