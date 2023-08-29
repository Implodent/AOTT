use core::marker::PhantomData;

use crate::{
        error::{Error, ParseResult, Simple},
        input::{Input, InputType, SliceInput},
        primitive::*,
        sync::RefC,
        *,
};

pub(crate) use private::*;

mod private {
        use crate::{
                input::{Input, InputType},
                IResult,
        };

        use super::{Parser, ParserExtras};

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
                fn invoke<'parse, I, O, E, P>(
                        parser: &P,
                        inp: Input<'parse, I, E>,
                ) -> IResult<'parse, I, E, Self::Output<O>>
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
                        f(x, y);
                }
                #[inline(always)]
                fn array<T, const N: usize>(x: [Self::Output<T>; N]) -> Self::Output<[T; N]> {
                        x
                }

                #[inline(always)]
                fn invoke<'parse, I, O, E, P>(
                        parser: &P,
                        inp: Input<'parse, I, E>,
                ) -> IResult<'parse, I, E, Self::Output<O>>
                where
                        I: InputType,
                        E: ParserExtras<I>,
                        P: Parser<I, O, E> + ?Sized,
                {
                        parser.parse(inp)
                }
        }

        /// Check mode - all output is discarded, and only uses parsers to check validity
        pub struct Check;

        impl Mode for Check {
                type Output<T> = ();
                #[inline(always)]
                fn bind<T, F: FnOnce() -> T>(_: F) -> Self::Output<T> {}
                #[inline(always)]
                fn map<T, U, F: FnOnce(T) -> U>((): Self::Output<T>, _: F) -> Self::Output<U> {}
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
                        (): Self::Output<T>,
                        (): Self::Output<U>,
                        _: F,
                ) -> Self::Output<V> {
                }
                #[inline(always)]
                fn combine_mut<T, U, F: FnOnce(&mut T, U)>(
                        _: &mut Self::Output<T>,
                        (): Self::Output<U>,
                        _: F,
                ) {
                }
                #[inline(always)]
                fn array<T, const N: usize>(_: [Self::Output<T>; N]) -> Self::Output<[T; N]> {}

                #[inline(always)]
                fn invoke<'parse, I, O, E, P>(
                        parser: &P,
                        inp: Input<'parse, I, E>,
                ) -> IResult<'parse, I, E, Self::Output<O>>
                where
                        I: InputType,
                        E: ParserExtras<I>,
                        P: Parser<I, O, E> + ?Sized,
                {
                        parser.check(inp)
                }
        }
}

pub trait Parser<I: InputType, O, E: ParserExtras<I>> {
        fn parse_from<'parse>(&self, input: &'parse I) -> IResult<'parse, I, E, O>
        where
                E: ParserExtras<I, Context = ()>,
        {
                self.parse(Input::new(input))
        }
        /// # Errors
        /// Returns an error if the parser failed.
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, O>;
        /// # Errors
        /// Returns an error if the parser failed.
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()>;

        fn or<P: Parser<I, O, E>>(self, other: P) -> Or<Self, P>
        where
                Self: Sized,
        {
                Or(self, other)
        }

        /// Transforms this parser's output with the `mapper` function.
        /// The `mapper` function cannot return an error. If you want to, consider using [`Parser::try_map`]
        ///
        /// # Example
        /// ```
        /// # use aott::prelude::*;
        /// let input = "hisnameisjuan";
        ///
        /// #[parser]
        /// fn parser(input: &str) -> (&'a str, &'a str) {
        ///     tuple((
        ///         choice((just("his"), just("her"))),
        ///         just("name"), just("is"),
        ///         any.repeated().slice(),
        ///         end
        ///     )).map(|(pronoun, _, _, name, _)| (pronoun, name)).parse(input)
        /// }
        /// let (pronoun, name) = parser.parse_from(&input).unwrap().1;
        /// assert_eq!(pronoun, "his");
        /// assert_eq!(name, "juan");
        /// ```
        fn map<U, F: Fn(O) -> U>(self, mapper: F) -> Map<Self, O, F, U>
        where
                Self: Sized,
        {
                Map(self, PhantomData, mapper, PhantomData)
        }
        fn to<U>(self, value: U) -> To<Self, O, U>
        where
                Self: Sized,
        {
                To(self, value, PhantomData)
        }
        fn ignored(self) -> Ignored<Self, O>
        where
                Self: Sized,
        {
                Ignored(self, EmptyPhantom::new())
        }
        fn try_map<F: Fn(U) -> Result<U, E::Error>, U>(self, f: F) -> TryMap<Self, F, O, U>
        where
                Self: Sized,
        {
                TryMap(self, f, PhantomData, PhantomData)
        }
        fn filter<F: Fn(&O) -> bool>(self, f: F) -> FilterParser<Self, F, O>
        where
                Self: Sized,
        {
                FilterParser(self, f, PhantomData)
        }
        fn repeated(self) -> Repeated<Self, O, Vec<O>>
        where
                Self: Sized,
        {
                Repeated {
                        parser: self,
                        phantom: PhantomData,
                }
        }
        fn repeated_custom<V: FromIterator<O>>(self) -> Repeated<Self, O, V>
        where
                Self: Sized,
        {
                Repeated {
                        parser: self,
                        phantom: PhantomData,
                }
        }
        fn slice<'a>(self) -> Slice<'a, I, E, O, Self>
        where
                I: SliceInput<'a>,
                Self: Sized,
        {
                slice(self)
        }
        fn then<O2, P: Parser<I, O2, E>>(self, other: P) -> Then<O, O2, Self, P, true>
        where
                Self: Sized,
        {
                Then(self, other, PhantomData)
        }
        fn ignore_then<O2, P: Parser<I, O2, E>>(self, other: P) -> Then<O, O2, Self, P, false>
        where
                Self: Sized,
        {
                Then(self, other, PhantomData)
        }
        fn then_ignore<O2, P: Parser<I, O2, E>>(self, other: P) -> Then<O2, O, P, Self, false>
        where
                Self: Sized,
        {
                Then(other, self, PhantomData)
        }
        fn optional(self) -> Maybe<Self>
        where
                Self: Sized,
        {
                Maybe(self)
        }

        #[doc(hidden)]
        fn boxed<'b>(self) -> Boxed<'b, I, O, E>
        where
                Self: MaybeSync + Sized + 'b,
        {
                Boxed {
                        inner: RefC::new(self),
                }
        }
}

impl<
                I: InputType,
                O,
                E: ParserExtras<I>,
                // only input
                F: for<'parse> Fn(Input<'parse, I, E>) -> IResult<'parse, I, E, O>,
        > Parser<I, O, E> for F
{
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, O> {
                self(input)
        }
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self(input).map(|(input, _)| (input, ()))
        }
}

pub trait ParserExtras<I: InputType> {
        type Error: Error<I>;
        type Context;
        // type State;

        fn recover<O>(
                _error: Self::Error,
                _context: Self::Context,
                input: I,
                prev_output: Option<O>,
                // Errors.secondary
                prev_errors: Vec<Self::Error>,
        ) -> ParseResult<I, O, Self::Error> {
                // default: noop
                ParseResult {
                        input,
                        output: prev_output,
                        errors: prev_errors,
                }
        }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct SimpleExtras<I: InputType, E: Error<I> = Simple<<I as InputType>::Token>>(
        PhantomData<I>,
        PhantomData<E>,
);

impl<I: InputType, E: Error<I>> ParserExtras<I> for SimpleExtras<I, E> {
        type Error = E;
        type Context = ();
}

/// See [`Parser::boxed`].
///
/// Due to current implementation details, the inner value is not, in fact, a [`Box`], but is an [`Rc`] to facilitate
/// efficient cloning. This is likely to change in the future. Unlike [`Box`], [`Rc`] has no size guarantees: although
/// it is *currently* the same size as a raw pointer.
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

impl<'b, I, O, E> Parser<I, O, E> for Boxed<'b, I, O, E>
where
        I: InputType,
        E: ParserExtras<I>,
{
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, O> {
                self.inner.parse(input)
        }
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.inner.check(input)
        }

        fn boxed<'c>(self) -> Boxed<'c, I, O, E>
        where
                Self: MaybeSync + Sized + 'c,
        {
                // Never double-box parsers
                self
        }
}

// MEH ITS A FUCKING FUNCTION GUHHHHH

// impl<I, O, E, T> Parser<I, O, E> for ::alloc::boxed::Box<T>
// where
//         I: InputType,
//         E: ParserExtras<I>,
//         T: Parser<I, O, E>,
// {
//         fn explode<'parse, M: Mode>(&self, inp: Input<I, E>) -> PResult<'parse, I, E, M, O>
//         where
//                 Self: Sized,
//         {
//                 T::explode::<M>(self, inp)
//         }

//         explode_extra!(O);
// }

impl<I, O, E, T> Parser<I, O, E> for ::alloc::rc::Rc<T>
where
        I: InputType,
        E: ParserExtras<I>,
        T: Parser<I, O, E>,
{
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.deref().check(input)
        }
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, O> {
                self.deref().parse(input)
        }
}

impl<I, O, E, T> Parser<I, O, E> for ::alloc::sync::Arc<T>
where
        I: InputType,
        E: ParserExtras<I>,
        T: Parser<I, O, E>,
{
        fn check<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                self.deref().check(input)
        }
        fn parse<'parse>(&self, input: Input<'parse, I, E>) -> IResult<'parse, I, E, O> {
                self.deref().parse(input)
        }
}
