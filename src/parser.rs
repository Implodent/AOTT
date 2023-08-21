use core::marker::PhantomData;

use crate::{
        error::{Error, ParseResult, Simple},
        input::{Input, InputType},
        sync::RefC,
        *,
};

pub(crate) use private::*;

mod private {
        use crate::{
                input::{Input, InputType},
                sync::{MaybeSync, RefC},
        };

        use super::{Boxed, Parser, ParserExtras};

        /// The result of calling [`Parser::explode`]
        pub type PResult<M, O> = Result<<M as Mode>::Output<O>, ()>;
        /// The result of calling [`IterParser::next`]
        pub type IPResult<M, O> = Result<Option<<M as Mode>::Output<O>>, ()>;

        /// An abstract parse mode - can be [`Emit`] or [`Check`] in practice, and represents the
        /// common interface for handling both in the same method.
        pub trait Mode {
                /// The output of this mode for a given type
                type Output<T>;

                /// Bind the result of a closure into an output
                fn bind<T, F: FnOnce() -> T>(f: F) -> Self::Output<T>;

                /// Given an [`Output`](Self::Output), takes its value and return a newly generated output
                fn map<T, U, F: FnOnce(T) -> U>(x: Self::Output<T>, f: F) -> Self::Output<U>;

                /// Choose between two fallible functions to execute depending on the mode.
                fn choose<A, T, E, F: FnOnce(A) -> Result<T, E>, G: FnOnce(A) -> Result<(), E>>(
                        arg: A,
                        f: F,
                        g: G,
                ) -> Result<Self::Output<T>, E>;

                /// Given two [`Output`](Self::Output)s, take their values and combine them into a new
                /// output value
                fn combine<T, U, V, F: FnOnce(T, U) -> V>(
                        x: Self::Output<T>,
                        y: Self::Output<U>,
                        f: F,
                ) -> Self::Output<V>;
                /// By-reference version of [`Mode::combine`].
                fn combine_mut<T, U, F: FnOnce(&mut T, U)>(
                        x: &mut Self::Output<T>,
                        y: Self::Output<U>,
                        f: F,
                );

                /// Given an array of outputs, bind them into an output of arrays
                fn array<T, const N: usize>(x: [Self::Output<T>; N]) -> Self::Output<[T; N]>;

                /// Invoke a parser user the current mode. This is normally equivalent to
                /// [`parser.explode::<M>(inp)`](Parser::explode), but it can be called on unsized values such as
                /// `dyn Parser`.
                fn invoke<I, O, E, P>(parser: &P, inp: &mut Input<I, E>) -> PResult<Self, O>
                where
                        I: InputType,
                        E: ParserExtras<I>,
                        P: Parser<I, O, E> + ?Sized;
        }

        /// Emit mode - generates parser output
        pub struct Emit;

        impl Mode for Emit {
                type Output<T> = T;
                #[inline(always)]
                fn bind<T, F: FnOnce() -> T>(f: F) -> Self::Output<T> {
                        f()
                }
                #[inline(always)]
                fn map<T, U, F: FnOnce(T) -> U>(x: Self::Output<T>, f: F) -> Self::Output<U> {
                        f(x)
                }
                #[inline(always)]
                fn choose<A, T, E, F: FnOnce(A) -> Result<T, E>, G: FnOnce(A) -> Result<(), E>>(
                        arg: A,
                        f: F,
                        _: G,
                ) -> Result<Self::Output<T>, E> {
                        f(arg)
                }
                #[inline(always)]
                fn combine<T, U, V, F: FnOnce(T, U) -> V>(
                        x: Self::Output<T>,
                        y: Self::Output<U>,
                        f: F,
                ) -> Self::Output<V> {
                        f(x, y)
                }
                #[inline(always)]
                fn combine_mut<T, U, F: FnOnce(&mut T, U)>(
                        x: &mut Self::Output<T>,
                        y: Self::Output<U>,
                        f: F,
                ) {
                        f(x, y)
                }
                #[inline(always)]
                fn array<T, const N: usize>(x: [Self::Output<T>; N]) -> Self::Output<[T; N]> {
                        x
                }

                #[inline(always)]
                fn invoke<I, O, E, P>(parser: &P, inp: &mut Input<I, E>) -> PResult<Self, O>
                where
                        I: InputType,
                        E: ParserExtras<I>,
                        P: Parser<I, O, E> + ?Sized,
                {
                        parser.explode_emit(inp)
                }
        }

        /// Check mode - all output is discarded, and only uses parsers to check validity
        pub struct Check;

        impl Mode for Check {
                type Output<T> = ();
                #[inline(always)]
                fn bind<T, F: FnOnce() -> T>(_: F) -> Self::Output<T> {}
                #[inline(always)]
                fn map<T, U, F: FnOnce(T) -> U>(_: Self::Output<T>, _: F) -> Self::Output<U> {}
                #[inline(always)]
                fn choose<A, T, E, F: FnOnce(A) -> Result<T, E>, G: FnOnce(A) -> Result<(), E>>(
                        arg: A,
                        _: F,
                        g: G,
                ) -> Result<Self::Output<T>, E> {
                        g(arg)
                }
                #[inline(always)]
                fn combine<T, U, V, F: FnOnce(T, U) -> V>(
                        _: Self::Output<T>,
                        _: Self::Output<U>,
                        _: F,
                ) -> Self::Output<V> {
                }
                #[inline(always)]
                fn combine_mut<T, U, F: FnOnce(&mut T, U)>(
                        _: &mut Self::Output<T>,
                        _: Self::Output<U>,
                        _: F,
                ) {
                }
                #[inline(always)]
                fn array<T, const N: usize>(_: [Self::Output<T>; N]) -> Self::Output<[T; N]> {}

                #[inline(always)]
                fn invoke<I, O, E, P>(parser: &P, inp: &mut Input<I, E>) -> PResult<Self, O>
                where
                        I: InputType,
                        E: ParserExtras<I>,
                        P: Parser<I, O, E> + ?Sized,
                {
                        parser.explode_check(inp)
                }
        }

        // TODO: Consider removing these sealed traits in favour of `Sealed`, with the given methods just being on `Parser`
        // with doc(hidden)

        pub trait ParserSealed<I: InputType, O, E: ParserExtras<I>> {
                fn explode<M: Mode>(&self, inp: &mut Input<I, E>) -> PResult<M, O>
                where
                        Self: Sized;

                fn explode_emit(&self, inp: &mut Input<I, E>) -> PResult<Emit, O>;
                fn explode_check(&self, inp: &mut Input<I, E>) -> PResult<Check, O>;

                fn boxed<'b>(self) -> Boxed<'b, I, O, E>
                where
                        Self: MaybeSync + Sized + 'b,
                {
                        Boxed {
                                inner: RefC::new(self),
                        }
                }
        }
}

pub trait Parser<I: InputType, O, E: ParserExtras<I>>: ParserSealed<I, O, E> {
        fn parse(&self, input: Input<I, E>) -> ParseResult<I, O, <E as ParserExtras<I>>::Error> {
                input.parse(self)
        }

        fn parse_with_extras(&self, input: I, context: E::Context);

        fn or<P: Parser<I, O, E>>(self, other: P) -> Or<Self, P>
        where
                Self: Sized,
        {
                Or(self, other)
        }
}

pub struct Or<A, B>(A, B);
impl<I: InputType + Clone, O, E: ParserExtras<I>, A: Parser<I, O, E>, B: Parser<I, O, E>>
        Parser<I, O, E> for Or<A, B>
where
        E::Context: Clone,
{
        fn parse_with_extras(
                &self,
                input: I,
                context: E::Context,
        ) -> ParseResult<I, O, <E as ParserExtras<I>>::Error> {
                self.0.parse_with_extras(input.clone(), context.clone())
                        .or(self.1.parse_with_extras(input, context))
        }
}

impl<A, B, I: InputType, O, E: ParserExtras<I>> ParserSealed<I, O, E> for Or<A, B>
where
        A: Parser<I, O, E>,
        B: Parser<I, O, E>,
{
        fn explode<M: Mode>(&self, inp: &mut Input<I, E>) -> PResult<M, O>
        where
                Self: Sized,
        {
                self.0.explode::<M>(inp).or(self.1.explode::<M>(inp))
        }

        fn boxed<'b>(self) -> Boxed<'b, I, O, E>
        where
                Self: MaybeSync + Sized + 'b,
        {
                Boxed {
                        inner: RefC::new(self),
                }
        }

        fn explode_check(&self, inp: &mut Input<I, E>) -> PResult<Check, O> {
                self.0.explode_check(inp).or(self.1.explode_check(inp))
        }

        fn explode_emit(&self, inp: &mut Input<I, E>) -> PResult<Emit, O> {
                self.0.explode_emit(inp).or(self.1.explode_emit(inp))
        }
}

impl<
                I: InputType,
                O,
                E: ParserExtras<I>,
                // only input
                F: Fn(&mut Input<I, E>) -> ParseResult<I, O, <E as ParserExtras<I>>::Error>,
        > Parser<I, O, E> for F
{
        fn parse_with_extras(
                &self,
                input: I,
                _context: E::Context,
        ) -> ParseResult<I, O, <E as ParserExtras<I>>::Error> {
                self(input)
        }
}

impl<
                I: InputType,
                O,
                E: ParserExtras<I>,
                // only input
                F: Fn(&mut Input<I, E>) -> ParseResult<I, O, <E as ParserExtras<I>>::Error>,
        > ParserSealed<I, O, E> for F
{
        fn boxed<'b>(self) -> Boxed<'b, I, O, E>
        where
                Self: MaybeSync + Sized + 'b,
        {
                Boxed {
                        inner: RefC::new(self),
                }
        }

        fn explode<M: Mode>(&self, inp: &mut Input<I, E>) -> PResult<M, O>
        where
                Self: Sized,
        {
                self(inp)
        }

        explode_extra!(O);
}

pub trait ParserExtras<I: InputType> {
        type Error: Error<I>;
        type Context;

        // fn recover<O>(
        //     _error: Self::Error,
        //     _context: Self::Context,
        //     input: I,
        //     prev_output: Option<O>,
        //     prev_errors: Vec<Self::Error>,
        // ) -> ParseResult<I, O, Self::Error> {
        //     // default: noop
        //     ParseResult {
        //         input,
        //         output: prev_output,
        //         errors: prev_errors,
        //     }
        // }
}

#[derive(Default, Clone, Copy)]
pub struct SimpleExtras<I, E = Simple<I>>(PhantomData<I>, PhantomData<E>);

impl<I: InputType, E: Error<I>> ParserExtras<I> for SimpleExtras<I, E> {
        type Error = E;
        type Context = ();
}
/// See [`Parser::boxed`].
///
/// Due to current implementation details, the inner value is not, in fact, a [`Box`], but is an [`Rc`] to facilitate
/// efficient cloning. This is likely to change in the future. Unlike [`Box`], [`Rc`] has no size guarantees: although
/// it is *currently* the same size as a raw pointer.
// TODO: Don't use an Rc
pub struct Boxed<'b, I: InputType, O, E: ParserExtras<I>> {
        inner: RefC<DynParser<'b, I, O, E>>,
}

impl<'b, I: InputType, O, E: ParserExtras<I>> Clone for Boxed<'b, I, O, E> {
        fn clone(&self) -> Self {
                Self {
                        inner: self.inner.clone(),
                }
        }
}

impl<'b, I, O, E> ParserSealed<I, O, E> for Boxed<'b, I, O, E>
where
        I: InputType,
        E: ParserExtras<I>,
{
        fn explode<M: Mode>(&self, inp: &mut Input<I, E>) -> PResult<M, O> {
                M::invoke(&*self.inner, inp)
        }

        fn boxed<'c>(self) -> Boxed<'c, I, O, E>
        where
                Self: MaybeSync + Sized + 'c,
        {
                // Never double-box parsers
                self
        }

        explode_extra!(O);
}

// impl<I, O, E, T> ParserSealed<I, O, E> for ::alloc::boxed::Box<T>
// where
//         I: InputType,
//         E: ParserExtras<I>,
//         T: Parser<I, O, E>,
// {
//         fn explode<M: Mode>(&self, inp: &mut Input<I, E>) -> PResult<M, O>
//         where
//                 Self: Sized,
//         {
//                 T::explode::<M>(self, inp)
//         }

//         explode_extra!(O);
// }

impl<I, O, E, T> ParserSealed<I, O, E> for ::alloc::rc::Rc<T>
where
        I: InputType,
        E: ParserExtras<I>,
        T: Parser<I, O, E>,
{
        fn explode<M: Mode>(&self, inp: &mut Input<I, E>) -> PResult<M, O>
        where
                Self: Sized,
        {
                T::explode::<M>(self, inp)
        }

        explode_extra!(O);
}

impl<I, O, E, T> ParserSealed<I, O, E> for ::alloc::sync::Arc<T>
where
        I: InputType,
        E: ParserExtras<I>,
        T: Parser<I, O, E>,
{
        fn explode<M: Mode>(&self, inp: &mut Input<I, E>) -> PResult<M, O>
        where
                Self: Sized,
        {
                T::explode::<M>(self, inp)
        }

        explode_extra!(O);
}
