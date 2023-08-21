use crate::{error::*, text::Char, MaybeRef};
use core::{
        borrow::Borrow,
        hash::Hash,
        marker::PhantomData,
        ops::{Range, RangeFrom},
};
/// A trait for types that represents a stream of input tokens. Unlike [`Iterator`], this type
/// supports backtracking and a few other features required by the crate.
///
/// `Input` abstracts over streams which yield tokens by value or reference, and which may or may not have
/// slices of tokens taken from them for use in parser output. There are multiple traits that inherit from
/// `Input` and are implemented by types to indicate support for these specific abilities. Various combinators
/// on the [`Parser`] trait may require that the input type implement one or more of these more specific traits.
///
/// Some common input types, and which traits they implement are:
/// - `&str`: [`SliceInput`], [`StrInput`], [`ValueInput`], [`ExactSizeInput`]
/// - `&[T]`: [`SliceInput`], [`ValueInput`], [`BorrowInput`], [`ExactSizeInput`]
/// - `Stream<I>`: [`ValueInput`], [`ExactSizeInput`] if `I: ExactSizeIterator`
pub trait Input {
        /// The type used to keep track of the current location in the stream
        #[doc(hidden)]
        type Offset: Copy + Hash + Ord + Into<usize>;

        /// The type of singular items read from the stream
        type Token;

        /// The type of a span on this input - to provide custom span context see [`Input::spanned`].
        type Span: Span;

        /// Get the offset representing the start of this stream
        #[doc(hidden)]
        fn start(&self) -> Self::Offset;

        /// The token type returned by [`Input::next_maybe`], allows abstracting over by-value and by-reference inputs.
        #[doc(hidden)]
        type TokenMaybe<'a>: Borrow<Self::Token> + Into<MaybeRef<'a, Self::Token>>;

        /// Get the next offset from the provided one, and the next token if it exists
        ///
        /// The token is effectively self-owning (even if it refers to the underlying input) so as to abstract over
        /// by-value and by-reference inputs. For alternatives with stronger guarantees, see [`ValueInput::next`] and
        /// `BorrowInput::next_ref`.
        ///
        /// # Safety
        ///
        /// `offset` must be generated by either `Input::start` or a previous call to this function.
        #[doc(hidden)]
        unsafe fn next_maybe<'a>(
                &'a self,
                offset: Self::Offset,
        ) -> (Self::Offset, Option<Self::TokenMaybe<'a>>);

        /// Create a span from a start and end offset.
        ///
        /// # Safety
        ///
        /// As with [`Input::next_maybe`], the offsets passed to this function must be generated by either [`Input::start`]
        /// or [`Input::next_maybe`].
        #[doc(hidden)]
        unsafe fn span(&self, range: Range<Self::Offset>) -> Self::Span;

        // Get the previous offset, saturating at zero
        #[doc(hidden)]
        fn prev(offs: Self::Offset) -> Self::Offset;

        /// Split an input that produces tokens of type `(T, S)` into one that produces tokens of type `T` and spans of
        /// type `S`.
        ///
        /// This is commonly required for lexers that generate token-span tuples. For example, `logos`'
        /// [`SpannedIter`](https://docs.rs/logos/0.12.0/logos/struct.Lexer.html#method.spanned) lexer generates such
        /// pairs.
        ///
        /// Also required is an 'End Of Input' (EoI) span. This span is arbitrary, but is used by the input to produce
        /// sensible spans that extend to the end of the input or are zero-width. Most implementations simply use some
        /// equivalent of `len..len` (i.e: a span where both the start and end offsets are set to the end of the input).
        /// However, what you choose for this span is up to you: but consider that the context, start, and end of the span
        /// will be recombined to create new spans as required by the parser.
        ///
        /// Although `Spanned` does implement [`BorrowInput`], please be aware that, as you might anticipate, the slices
        /// will be those of the original input (usually `&[(T, S)]`) and not `&[T]` so as to avoid the need to copy
        /// around sections of the input.
        fn spanned<T, S>(self, eoi: S) -> SpannedInput<T, S, Self>
        where
                Self: Input<Token = (T, S)> + Sized,
                S: Span + Clone,
        {
                SpannedInput {
                        input: self,
                        eoi,
                        phantom: PhantomData,
                }
        }

        /// Add extra context to spans generated by this input.
        ///
        /// This is useful if you wish to include extra context that applies to all spans emitted during a parse, such as
        /// an identifier that corresponds to the file the spans originated from.
        fn with_context<C>(self, context: C) -> WithContext<C, Self>
        where
                Self: Sized,
                C: Clone,
                Self::Span: Span<Context = ()>,
        {
                WithContext {
                        input: self,
                        context,
                }
        }
}
/// Implement by inputs that have a known size (including spans)
pub trait ExactSizeInput: Input {
        /// Get a span from a start offset to the end of the input.
        #[doc(hidden)]
        unsafe fn span_from(&self, range: RangeFrom<Self::Offset>) -> Self::Span;
}

/// Implemented by inputs that represent slice-like streams of input tokens.
pub trait SliceInput<'a>: ExactSizeInput {
        /// The unsized slice type of this input. For [`&str`] it's `&str`, and for [`&[T]`] it will be `&[T]`.
        type Slice: 'a;

        /// Get a slice from a start and end offset
        // TODO: Make unsafe
        #[doc(hidden)]
        fn slice(&self, range: Range<Self::Offset>) -> Self::Slice;

        /// Get a slice from a start offset till the end of the input
        // TODO: Make unsafe
        #[doc(hidden)]
        fn slice_from(&self, from: RangeFrom<Self::Offset>) -> Self::Slice;
}

// Implemented by inputs that reference a string slice and use byte indices as their offset.
/// A trait for types that represent string-like streams of input tokens
pub trait StrInput<'a, C: Char>:
        ValueInput<Offset = usize, Token = C> + SliceInput<'a, Slice = &'a C::Str>
{
}

/// Implemented by inputs that can have tokens borrowed from them.
pub trait ValueInput: Input {
        /// Get the next offset from the provided one, and the next token if it exists
        ///
        /// # Safety
        ///
        /// `offset` must be generated by either `Input::start` or a previous call to this function.
        #[doc(hidden)]
        unsafe fn next(&self, offset: Self::Offset) -> (Self::Offset, Option<Self::Token>);
}

/// Implemented by inputs that can have tokens borrowed from them.
pub trait BorrowInput<'a>: Input {
        /// Borrowed version of [`ValueInput::next`] with the same safety requirements.
        ///
        /// # Safety
        ///
        /// Same as [`ValueInput::next`]
        #[doc(hidden)]
        unsafe fn next_ref(&self, offset: Self::Offset) -> (Self::Offset, Option<&'a Self::Token>);
}

/// A wrapper around an input that splits an input into spans and tokens. See [`Input::spanned`].
#[derive(Copy, Clone)]
pub struct SpannedInput<T, S, I> {
        input: I,
        eoi: S,
        phantom: PhantomData<T>,
}

// guhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh
/// Utility type required to allow [`SpannedInput`] to implement [`Input`].
#[doc(hidden)]
pub struct SpannedTokenMaybe<'a, I: Input, T, S>(I::TokenMaybe<'a>, PhantomData<(T, S)>);

impl<'a, I: Input<Token = (T, S)>, T, S> Borrow<T> for SpannedTokenMaybe<'a, I, T, S> {
        #[inline(always)]
        fn borrow(&self) -> &T {
                &self.0.borrow().0
        }
}

impl<'a, I: Input<Token = (T, S)>, T, S: 'a> From<SpannedTokenMaybe<'a, I, T, S>>
        for MaybeRef<'a, T>
{
        #[inline(always)]
        fn from(st: SpannedTokenMaybe<'a, I, T, S>) -> MaybeRef<'a, T> {
                match st.0.into() {
                        MaybeRef::Ref((tok, _)) => MaybeRef::Ref(tok),
                        MaybeRef::Val((tok, _)) => MaybeRef::Val(tok),
                }
        }
}

impl<T, S: Span + Clone, I: Input<Token = (T, S)>> Input for SpannedInput<T, S, I> {
        type Token = T;
        type Offset = I::Offset;
        type Span = S;
        type TokenMaybe<'a> = SpannedTokenMaybe<'a, I, T, S>;

        #[inline(always)]
        fn start(&self) -> Self::Offset {
                self.input.start()
        }

        #[inline(always)]
        unsafe fn next_maybe<'a>(
                &'a self,
                offset: Self::Offset,
        ) -> (Self::Offset, Option<Self::TokenMaybe<'a>>) {
                let (offset, tok) = self.input.next_maybe(offset);
                (offset, tok.map(|tok| SpannedTokenMaybe(tok, PhantomData)))
        }

        #[inline(always)]
        unsafe fn span(&self, range: Range<Self::Offset>) -> Self::Span {
                let start = self
                        .input
                        .next_maybe(range.start)
                        .1
                        .map_or(self.eoi.start(), |tok| tok.borrow().1.start());
                let end = self
                        .input
                        .next_maybe(I::prev(range.end))
                        .1
                        .map_or(self.eoi.start(), |tok| tok.borrow().1.end());
                S::new(self.eoi.context(), start..end)
        }

        #[inline(always)]
        fn prev(offs: Self::Offset) -> Self::Offset {
                I::prev(offs)
        }
}