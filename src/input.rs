#![allow(dead_code)]

#[cfg(feature = "builtin-text")]
use crate::text::Char;
use crate::{
        error::{Error, Located},
        parser::{Parser, ParserExtras},
};
use std::{
        fmt::Display,
        hash::Hash,
        marker::PhantomData,
        ops::{Range, RangeFrom},
};

pub trait Span {
        type Offset: Clone + Display;
        type Context: Clone;

        fn new(context: Self::Context, range: Range<Self::Offset>) -> Self
        where
                Self: Sized;

        fn start(&self) -> Self::Offset;

        fn end(&self) -> Self::Offset;

        fn range(&self) -> Range<Self::Offset> {
                self.start()..self.end()
        }

        fn context(&self) -> Self::Context;
}

impl<O: Clone + Display> Span for Range<O> {
        type Offset = O;

        type Context = ();

        fn new((): Self::Context, range: Range<Self::Offset>) -> Self
        where
                Self: Sized,
        {
                range
        }

        fn start(&self) -> Self::Offset {
                self.start.clone()
        }

        fn end(&self) -> Self::Offset {
                self.end.clone()
        }

        fn context(&self) -> Self::Context {}
}

#[allow(clippy::module_name_repetitions)]
pub trait InputType {
        /// The token type that this input returns.
        /// For `&str` this is `char`, for `&[T]` this is `T`, etc.
        type Token;

        /// The owned type containing Self that allows mutation.
        /// For `&str` this is [`String`], for `&[T]` this is [`Vec<T>`], etc.
        #[doc(hidden)]
        type OwnedMut;

        /// The span that this input gives.
        type Span: Span;

        /// The offset of this input. Usually `usize`.
        type Offset: Into<usize> + Copy + Hash + Ord + Eq;

        #[doc(hidden)]
        fn start(&self) -> Self::Offset;

        /// Gets the next token.
        ///
        /// # Safety
        /// If `offset` is not strictly the one provided by `Self::start` or returned as the first tuple value from this function,
        /// calling `next` is undefined behavior. It may index memory outside of the desired range, it may segfault, it may panic etc. etc.
        #[doc(hidden)]
        unsafe fn next(&self, offset: Self::Offset) -> (Self::Offset, Option<Self::Token>);

        /// Gives the offset before the given `offset`.
        /// Usually this uses `saturating_sub` on the concrete offset type (for example `usize`).
        #[doc(hidden)]
        fn prev(&self, offset: Self::Offset) -> Self::Offset;

        /// Converts a range of `Self::Offset` to the span type of this input.
        fn span(&self, span: Range<Self::Offset>) -> Self::Span;

        fn spanned<T, S: Span + Clone>(self, eoi: S) -> SpannedInput<T, S, Self>
        where
                Self: InputType<Token = (T, S)> + Sized,
        {
                SpannedInput {
                        input: self,
                        eoi,
                        phantom: PhantomData,
                }
        }
}

/// A wrapper around an input that splits an input into spans and tokens. See [`Input::spanned`].
#[derive(Copy, Clone)]
pub struct SpannedInput<T, S, I> {
        input: I,
        eoi: S,
        phantom: PhantomData<T>,
}

impl<T, S: Span + Clone, I: InputType<Token = (T, S)>> InputType for SpannedInput<T, S, I> {
        type Offset = I::Offset;
        type Token = T;
        type Span = S;
        type OwnedMut = I::OwnedMut;

        #[inline(always)]
        fn start(&self) -> Self::Offset {
                self.input.start()
        }

        unsafe fn next(&self, offset: Self::Offset) -> (Self::Offset, Option<Self::Token>) {
                let (offset, token) = self.input.next(offset);
                (offset, token.map(|(token, _)| token))
        }

        fn span(&self, range: Range<Self::Offset>) -> Self::Span {
                let start = unsafe { self.input.next(range.start) }
                        .1
                        .map_or(self.eoi.start(), |tok| tok.1.start());
                let end = unsafe { self.input.next(self.input.prev(range.end)) }
                        .1
                        .map_or(self.eoi.start(), |tok| tok.1.end());
                S::new(self.eoi.context(), start..end)
        }

        fn prev(&self, offset: Self::Offset) -> Self::Offset {
                self.input.prev(offset)
        }
}

impl<'a> InputType for &'a str {
        type Token = char;
        type OwnedMut = String;
        type Offset = usize;
        type Span = Range<usize>;

        fn span(&self, span: Range<Self::Offset>) -> Self::Span {
                span
        }

        #[inline]
        fn start(&self) -> usize {
                0
        }

        fn prev(&self, offset: usize) -> usize {
                offset.saturating_sub(1)
        }

        #[inline(always)]
        unsafe fn next(&self, offset: usize) -> (usize, Option<Self::Token>) {
                if offset < self.len() {
                        // SAFETY: `offset < self.len()` above guarantees offset is in-bounds
                        //         We only ever return offsets that are at a character boundary
                        let c = unsafe {
                                self.get_unchecked(offset..)
                                        .chars()
                                        .next()
                                        .unwrap_unchecked()
                        };
                        (offset + c.len_utf8(), Some(c))
                } else {
                        (offset, None)
                }
        }
}

impl<'a, T: Clone> InputType for &'a [T] {
        type Token = T;
        type OwnedMut = Vec<T>;
        type Offset = usize;
        type Span = Range<usize>;

        fn span(&self, span: Range<Self::Offset>) -> Self::Span {
                span
        }

        unsafe fn next(&self, offset: usize) -> (usize, Option<Self::Token>) {
                if offset < self.len() {
                        // SAFETY: `offset < self.len()` above guarantees offset is in-bounds
                        //         We only ever return offsets that are at a character boundary
                        let tok = unsafe { self.get_unchecked(offset) };
                        (offset + 1, Some(tok.clone()))
                } else {
                        (offset, None)
                }
        }

        fn start(&self) -> usize {
                0
        }

        fn prev(&self, offset: usize) -> usize {
                offset.saturating_sub(1)
        }
}

#[doc(hidden)]
pub trait ExactSizeInput: InputType {
        unsafe fn span_from(&self, range: RangeFrom<usize>) -> Range<usize>;
}

#[doc(hidden)]
#[derive(Debug)]
pub struct Errors<T, E> {
        pub alt: Option<Located<T, E>>,
        pub secondary: Vec<Located<T, E>>,
}

impl<T, E> Default for Errors<T, E> {
        fn default() -> Self {
                Self {
                        alt: None,
                        secondary: vec![],
                }
        }
}

pub struct InputOwned<I: InputType, E: ParserExtras<I>> {
        #[doc(hidden)]
        pub input: I,
        #[doc(hidden)]
        pub cx: E::Context,
        #[doc(hidden)]
        pub errors: Errors<usize, E::Error>,
}

impl<I: InputType, E: ParserExtras<I>> InputOwned<I, E> {
        pub fn from_input_with_context(input: I, context: E::Context) -> Self {
                Self {
                        input,
                        cx: context,
                        errors: Errors::default(),
                }
        }
        pub fn from_input(input: I) -> Self
        where
                E::Context: Default,
        {
                Self {
                        input,
                        cx: E::Context::default(),
                        errors: Errors::default(),
                }
        }
        pub fn as_ref_at_zero(&mut self) -> Input<'_, I, E> {
                Input {
                        offset: self.input.start(),
                        input: &self.input,
                        cx: &self.cx,
                }
        }

        pub unsafe fn as_ref_at(&mut self, offset: I::Offset) -> Input<'_, I, E> {
                Input {
                        offset,
                        input: &self.input,
                        cx: &self.cx,
                }
        }
}

/// **Warning** `InputOwned` and `Input` are an unstable & internal API.
/// This could change at any time without notice.
/// Please consider using primitives like `any` over functions in this struct. Please.
/// If you do, support is not guaranteed.
/// Changing the `offset` to arbitrary values could lead to undefined behavior. Don't modify anything in this struct if you want to be free of UB and/or segfaults.
#[derive(Debug)]
pub struct Input<'parse, I: InputType, E: ParserExtras<I>> {
        #[doc(hidden)]
        pub offset: I::Offset,
        #[doc(hidden)]
        pub input: &'parse I,
        // #[doc(hidden)]
        // pub errors: &'parse mut Errors<usize, E::Error>,
        #[doc(hidden)]
        pub cx: &'parse E::Context,
}

impl<'parse, I: InputType, E: ParserExtras<I, Context = ()>> Input<'parse, I, E> {
        pub fn new(input: &'parse I) -> Self {
                Self {
                        offset: input.start(),
                        input,
                        cx: &(),
                }
        }
}

impl<'parse, I: InputType, E: ParserExtras<I>> Input<'parse, I, E> {
        pub fn new_with_context(input: &'parse I, cx: &'parse E::Context) -> Self
        where
                E: ParserExtras<I>,
        {
                Self {
                        offset: input.start(),
                        input,
                        cx,
                }
        }

        /// Returns the context of the input.
        pub fn context(&self) -> &'parse E::Context {
                self.cx
        }

        #[inline(always)]
        pub(crate) fn skip_while(&mut self, f: &impl Fn(&I::Token) -> bool) {
                loop {
                        // SAFETY: offset was generated by previous call to `Input::next`
                        let (offset, token) = unsafe { self.input.next(self.offset) };
                        if token.filter(f).is_none() {
                                break;
                        }
                        self.offset = offset;
                }
        }

        #[inline(always)]
        pub(crate) fn next_inner(&mut self) -> (I::Offset, Option<I::Token>) {
                // SAFETY: offset was generated by previous call to `Input::next`
                let (offset, token) = unsafe { self.input.next(self.offset) };
                self.offset = offset;
                (offset, token)
        }

        /// Invokes `parser` with this input.
        pub fn parse<O, P: Parser<I, O, E> + ?Sized>(&mut self, parser: &P) -> Result<O, E::Error> {
                parser.parse_with(self)
        }

        /// Invokes `parser` in check mode (not emitting an output, just checking for errors) with this input.
        pub fn check<O, P: Parser<I, O, E>>(&mut self, parser: &P) -> Result<(), E::Error> {
                parser.check_with(self)
        }

        /// Save the current parse state as a [`Marker`].
        ///
        /// You can rewind back to this state later with [`Self::rewind`].
        #[inline(always)]
        pub fn save(&self) -> Marker<I> {
                Marker {
                        offset: self.offset,
                        err_count: 0, //self.errors.secondary.len(),
                }
        }

        /// Reset the parse state to that represented by the given [`Marker`].
        ///
        /// You can create a marker with which to perform rewinding using [`Self::save`].
        #[inline(always)]
        pub fn rewind(&mut self, marker: Marker<I>) {
                // self.errors.secondary.truncate(marker.err_count);
                self.offset = marker.offset;
        }

        #[inline(always)]
        pub fn offset(&self) -> I::Offset {
                self.offset
        }

        /// Get the next token in the input by value. Returns `None` if the end of the input has been reached.
        #[inline(always)]
        pub fn next_or_none(&mut self) -> Option<I::Token> {
                self.next_inner().1
        }

        /// Peek the next token in the input. Returns `Err(UnexpectedEOF)` if the end of the input has been reached.
        #[inline(always)]
        pub fn peek(&self) -> Result<I::Token, E::Error> {
                let befunge = self.offset;
                // SAFETY: offset was generated by previous call to `Input::next`
                unsafe { self.input.next(self.offset).1 }
                        .ok_or_else(|| Error::unexpected_eof(self.span_since(befunge), None))
        }
        #[inline(always)]
        pub fn span_since(&self, before: I::Offset) -> I::Span {
                self.input.span(before..self.offset)
        }
        #[inline(always)]
        pub fn next(&mut self) -> Result<I::Token, E::Error> {
                let befunge = self.offset;
                self.next_or_none()
                        .ok_or_else(|| Error::unexpected_eof(self.span_since(befunge), None))
        }
        #[inline(always)]
        pub fn skip(&mut self) -> Result<(), E::Error> {
                let before = self.offset;
                self.offset = Some(unsafe { self.input.next(self.offset) })
                        .and_then(|x| x.1.map(|_| x.0))
                        .ok_or_else(|| Error::unexpected_eof(self.span_since(before), None))?;

                Ok(())
        }
        #[inline(always)]
        pub fn current(&self) -> Option<I::Token> {
                unsafe { self.input.next(self.input.prev(self.offset)) }.1
        }

        #[inline(always)]
        pub fn slice(&self, range: Range<I::Offset>) -> I::Slice
        where
                I: SliceInput<'parse>,
        {
                self.input.slice(self.input.span(range))
        }

        #[inline(always)]
        pub fn full_slice(&self) -> I::Slice
        where
                I: SliceInput<'parse>,
        {
                self.input.full_slice()
        }

        #[inline(always)]
        pub fn slice_since(&self, before: I::Offset) -> I::Slice
        where
                I: SliceInput<'parse>,
        {
                self.input.slice(self.span_since(before))
        }

        #[inline(always)]
        pub fn with_context<E2: ParserExtras<I>>(
                &self,
                cx: &'parse E2::Context,
        ) -> Input<'parse, I, E2> {
                Input {
                        input: self.input,
                        cx,
                        offset: self.offset,
                }
        }

        #[inline(always)]
        pub fn no_context<E2: ParserExtras<I, Context = ()>>(&self) -> Input<'parse, I, E2> {
                Input {
                        input: self.input,
                        cx: &(),
                        offset: self.offset,
                }
        }

        #[inline(always)]
        pub fn parse_with_context<E2: ParserExtras<I>, O>(
                &mut self,
                cx: &'parse E2::Context,
                parser: impl Parser<I, O, E2>,
        ) -> Result<O, E2::Error> {
                let mut input = self.with_context(cx);

                let result = input.parse(&parser);

                self.offset = input.offset;

                result
        }

        #[inline(always)]
        pub fn parse_no_context<E2: ParserExtras<I, Context = ()>, O>(
                &mut self,
                parser: impl Parser<I, O, E2>,
        ) -> Result<O, E2::Error> {
                let mut input = self.no_context();

                let result = input.parse(&parser);

                self.offset = input.offset;

                result
        }
}

#[derive(Debug)]
pub struct Marker<I: InputType> {
        pub offset: I::Offset,
        err_count: usize,
}

impl<I: InputType> Clone for Marker<I> {
        fn clone(&self) -> Self {
                Self {
                        offset: self.offset,
                        err_count: self.err_count,
                }
        }
}

impl<I: InputType> Copy for Marker<I> {}

/// Implemented by inputs that represent slice-like streams of input tokens.
pub trait SliceInput<'a>: ExactSizeInput {
        /// The sliced type of this input. For [`&str`] it's `&'a str`, and for [`&[T]`] it will be `&'a [T]`.
        type Slice: 'a;

        /// Get the full slice of the input
        #[doc(hidden)]
        fn full_slice(&self) -> Self::Slice;

        /// Get a slice from a start and end offset
        #[doc(hidden)]
        fn slice(&self, range: Self::Span) -> Self::Slice;

        /// Get a slice from a start offset till the end of the input
        #[doc(hidden)]
        fn slice_from(&self, from: RangeFrom<Self::Offset>) -> Self::Slice;
}

#[cfg(feature = "builtin-text")]
pub trait StrInput<'a, C: Char>: InputType<Token = C> + SliceInput<'a, Slice = &'a C::Str> {}
impl<'a> ExactSizeInput for &'a str {
        #[inline(always)]
        unsafe fn span_from(&self, range: RangeFrom<usize>) -> Range<usize> {
                range.start..self.len()
        }
}
impl<'a, T: Clone> ExactSizeInput for &'a [T] {
        #[inline(always)]
        unsafe fn span_from(&self, range: RangeFrom<usize>) -> Range<usize> {
                range.start..self.len()
        }
}
// impl<'a, T: Clone + 'a, const N: usize> ExactSizeInput for &'a [T; N] {
//         #[inline(always)]
//         unsafe fn span_from(&self, range: RangeFrom<usize>) -> Range<usize> {
//                 (range.start..N).into()
//         }
// }
#[cfg(feature = "builtin-text")]
impl<'a> StrInput<'a, char> for &'a str {}

impl<'a> SliceInput<'a> for &'a str {
        type Slice = &'a str;

        #[inline(always)]
        fn full_slice(&self) -> Self::Slice {
                *self
        }

        #[inline(always)]
        fn slice(&self, range: Range<usize>) -> Self::Slice {
                &self[range]
        }

        #[inline(always)]
        fn slice_from(&self, from: RangeFrom<usize>) -> Self::Slice {
                &self[from]
        }
}

#[cfg(feature = "builtin-text")]
impl<'a> StrInput<'a, u8> for &'a [u8] {}

impl<'a, T: Clone> SliceInput<'a> for &'a [T] {
        type Slice = &'a [T];

        #[inline(always)]
        fn full_slice(&self) -> Self::Slice {
                *self
        }

        #[inline(always)]
        fn slice(&self, range: Range<usize>) -> Self::Slice {
                &self[range]
        }

        #[inline(always)]
        fn slice_from(&self, from: RangeFrom<usize>) -> Self::Slice {
                &self[from]
        }
}
