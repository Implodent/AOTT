use core::{marker::PhantomData, ops::Range};

#[allow(deprecated)]
use crate::{
        error::{Error, ParseResult},
        input::{Input, InputType, SliceInput},
        primitive::*,
        sync::RefC,
        *,
};

pub use mode::*;

pub mod mode {
        use crate::{
                input::{Input, InputType},
                PResult,
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

                /// Invoke a function if [`Self::Output`] can be coerced to `T` (Emit mode), or just don't (Check mode).
                fn invoke_unbind<T, F: FnMut(T)>(f: F, output: Self::Output<T>);

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

                fn invoke<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>>(
                        parser: &P,
                        input: &mut Input<I, E>,
                ) -> PResult<I, Self::Output<O>, E>;
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
                fn invoke_unbind<T, F: FnMut(T)>(mut f: F, output: Self::Output<T>) {
                        f(output);
                }
                #[inline(always)]
                fn invoke<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>>(
                        parser: &P,
                        input: &mut Input<I, E>,
                ) -> PResult<I, Self::Output<O>, E> {
                        parser.parse_with(input)
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
                fn invoke_unbind<T, F: FnMut(T)>(_: F, (): Self::Output<T>) {}
                #[inline(always)]
                fn combine_mut<T, U, F: FnOnce(&mut T, U)>(
                        _: &mut Self::Output<T>,
                        (): Self::Output<U>,
                        _: F,
                ) {
                }
                #[inline(always)]
                fn array<T, const N: usize>(_: [Self::Output<T>; N]) -> Self::Output<[T; N]> {}
                fn invoke<I: InputType, O, E: ParserExtras<I>, P: Parser<I, O, E>>(
                        parser: &P,
                        input: &mut Input<I, E>,
                ) -> PResult<I, Self::Output<O>, E> {
                        parser.check_with(input)
                }
        }
}

pub trait Parser<I: InputType, O, E: ParserExtras<I>> {
        /// Invokes this parser on this input.
        /// # Errors
        /// Returns an error if the parser failed.
        fn parse(&self, input: I) -> PResult<I, O, E>
        where
                E: ParserExtras<I, Context = ()>,
        {
                let mut input = Input::new(&input);
                self.parse_with(&mut input)
        }

        /// Invokes this parser on this input.
        /// # Errors
        /// Returns an error if the parser failed.
        fn parse_with_context(&self, input: I, context: E::Context) -> PResult<I, O, E> {
                let mut input = Input::new_with_context(&input, &context);
                self.parse_with(&mut input)
        }

        /// Runs the parser logic, producing an output, or an error.
        /// # Errors
        /// Returns an error if the parser failed.
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O, E>;
        /// Runs the parser logic without producing output, thus significantly reducing the number of allocations.
        /// # Errors
        /// Returns an error if the parser failed.
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E>;

        /// Transform this parser to try and invoke the `other` parser on failure, and if that one fails, fail too.
        /// If you are chaining a lot of [`or`](`Parser::or`) calls, please consider using [`choice`].
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
        /// #[aott::derive::parser]
        /// fn parser(input: &str) -> (&'a str, &'a str) {
        ///     (
        ///         choice((just("his"), just("her"))),
        ///         just("name"), just("is"),
        ///         any.repeated().slice(),
        ///         end
        ///     ).map(|(pronoun, _, _, name, _)| (pronoun, name)).parse_with(input)
        /// }
        /// assert_eq!(parser.parse(input), Ok(("his", "juan")));
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
        fn try_map<F: Fn(O) -> Result<U, E::Error>, U>(self, f: F) -> TryMap<Self, F, O, U>
        where
                Self: Sized,
        {
                TryMap(self, f, PhantomData, PhantomData)
        }
        fn try_map_with_span<F: Fn(O, Range<I::Offset>) -> Result<U, E::Error>, U>(
                self,
                f: F,
        ) -> TryMapWithSpan<Self, F, O, U>
        where
                Self: Sized,
        {
                TryMapWithSpan(self, f, PhantomData, PhantomData)
        }
        fn filter<F: Fn(&O) -> bool>(self, f: F) -> FilterParser<Self, F, O>
        where
                Self: Sized,
        {
                FilterParser(self, f, PhantomData)
        }

        /// # Example
        /// ```
        /// # use aott::prelude::*;
        /// use aott::text::Char;
        /// let parser = filter::<&str, extra::Err<&str>>(|c: &char| c.is_ident_start()).then(filter(|c: &char| c.is_ident_continue()).repeated());
        /// assert_eq!(parser.parse("hello"), Ok(('h', "ello".chars().collect())));
        /// ```
        fn repeated(self) -> Repeated<Self, O, Vec<O>>
        where
                Self: Sized,
        {
                Repeated {
                        parser: self,
                        phantom: PhantomData,
                        at_least: 0,
                        at_most: !0,
                }
        }
        fn repeated_custom<V: FromIterator<O>>(self) -> Repeated<Self, O, V>
        where
                Self: Sized,
        {
                Repeated {
                        parser: self,
                        phantom: PhantomData,
                        at_least: 0,
                        at_most: !0,
                }
        }

        fn separated_by<O2, A: Parser<I, O2, E>>(
                self,
                delimiter: A,
        ) -> SeparatedBy<Self, A, O, O2, Vec<O>>
        where
                Self: Sized,
        {
                SeparatedBy {
                        allow_leading: false,
                        allow_trailing: false,
                        at_least: 0,
                        at_most: !0,
                        delimiter,
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
        fn then<O2, P: Parser<I, O2, E>>(self, other: P) -> Then<O, O2, Self, P, true, false>
        where
                Self: Sized,
        {
                Then(self, other, PhantomData)
        }
        fn ignore_then<O2, P: Parser<I, O2, E>>(self, other: P) -> Then<O, O2, Self, P, false, true>
        where
                Self: Sized,
        {
                Then(self, other, PhantomData)
        }
        fn then_ignore<O2, P: Parser<I, O2, E>>(
                self,
                other: P,
        ) -> Then<O, O2, Self, P, false, false>
        where
                Self: Sized,
        {
                Then(self, other, PhantomData)
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
                F: Fn(&mut Input<I, E>) -> PResult<I, O, E>,
        > Parser<I, O, E> for F
{
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O, E> {
                self(input)
        }
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self(input).map(|_| {})
        }
}

pub trait ParserExtras<I: InputType> {
        type Error: Error<I>;
        type Context;
        // type State;

        #[deprecated(since = "0.3.0", note = "together with ParseResult")]
        #[allow(deprecated)]
        fn recover<O>(
                _error: Self::Error,
                _context: Self::Context,
                input: Input<'_, I, Self>,
                prev_output: Option<O>,
                // Errors.secondary
                prev_errors: Vec<Self::Error>,
        ) -> ParseResult<'_, I, O, Self>
        where
                Self: Sized,
        {
                // default: noop
                ParseResult {
                        input,
                        output: prev_output,
                        errors: prev_errors,
                }
        }
}

/// See [`Parser::boxed`].
///
/// Due to current implementation details, the inner value is not, in fact, a [`Box`], but is an [`Rc`] to facilitate
/// efficient cloning. This is likely to change in the future. Unlike [`Box`], [`Rc`] has no size guarantees: although
/// it is *currently* the same size as a raw pointer.
///
/// [`Rc`]: `alloc::rc::Rc`
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
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O, E> {
                self.inner.parse_with(input)
        }
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.inner.check_with(input)
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
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.deref().check_with(input)
        }
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O, E> {
                self.deref().parse_with(input)
        }
}

impl<I, O, E, T> Parser<I, O, E> for ::alloc::sync::Arc<T>
where
        I: InputType,
        E: ParserExtras<I>,
        T: Parser<I, O, E>,
{
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.deref().check_with(input)
        }
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O, E> {
                self.deref().parse_with(input)
        }
}
