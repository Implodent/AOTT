use core::{mem::MaybeUninit, ops::RangeTo};

use crate::{pfn_type, MaybeUninitExt};

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
) -> pfn_type!(I, Vec<I::Token>, E) {
        move |input| {
                let before = input.offset;
                let range = amount.range();
                let mut result = vec![];
                let mut n = 0usize;
                loop {
                        if range.end < n {
                                break;
                        }

                        n += 1;
                        result.push(input.next()?);
                }
                if n < range.start {
                        let error = Error::expected_token_found(
                                Span::new_usize(input.span_since(before)),
                                vec![],
                                crate::MaybeDeref::Val(unsafe {
                                        input.input.next(before).1.expect("no token??")
                                }),
                        );
                        Err(error)
                } else {
                        Ok(result)
                }
        }
}

pub struct TakeExact<const A: usize>(usize);
impl<I: InputType, E: ParserExtras<I>, const A: usize> Parser<I, [I::Token; A], E>
        for TakeExact<A>
{
        fn parse_with<'parse>(&self, inp: &mut Input<I, E>) -> PResult<I, [I::Token; A], E> {
                let mut result: [MaybeUninit<I::Token>; A] = MaybeUninitExt::uninit_array();

                for i in 0..A {
                        result[i] = MaybeUninit::new(inp.next()?);
                }

                Ok(unsafe { MaybeUninitExt::array_assume_init(result) })
        }
        fn check_with(&self, inp: &mut Input<I, E>) -> PResult<I, (), E> {
                for _ in 0..A {
                        inp.next()?;
                }

                Ok(())
        }
}

#[must_use]
pub fn take_exact<const A: usize>() -> TakeExact<A> {
        TakeExact(A)
}
