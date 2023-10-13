#![allow(dead_code)]
use crate::{
        error::{FundamentalError as Error, Located},
        extra,
        parser::{Parser, ParserExtras},
        text::Char,
};
use core::ops::{Range, RangeFrom};

#[allow(clippy::module_name_repetitions)]
pub trait InputType {
        type Token;

        /// The owned type containing Self
        #[doc(hidden)]
        type Owned: AsRef<Self>;

        #[doc(hidden)]
        fn start(&self) -> usize;

        /// # Safety
        /// If `offset` is not strictly the one provided by `Self::start` or returned as the first tuple value from this function,
        /// calling `next` is undefined behavior. It may index memory outside of the desired range, it may segfault, it may panic etc. etc.
        /// Stay safe and don't use this api unless you want to explode.
        #[doc(hidden)]
        unsafe fn next(&self, offset: usize) -> (usize, Option<Self::Token>);

        #[doc(hidden)]
        fn prev(offset: usize) -> usize;

        fn has_tokens_left(&self, offset: usize) -> Option<bool>;
}

impl InputType for str {
        type Token = char;

        type Owned = String;

        #[inline]
        fn start(&self) -> usize {
                0
        }

        fn prev(offset: usize) -> usize {
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

        fn has_tokens_left(&self, offset: usize) -> Option<bool> {
                Some(offset < self.len())
        }
}

impl<T: Clone> InputType for [T] {
        type Token = T;
        type Owned = Vec<T>;

        unsafe fn next(&self, offset: usize) -> (usize, Option<Self::Token>) {
                if offset < self.len() {
                        // SAFETY: `offset < self.len()` above guarantees offset is in-bounds
                        //         We only ever return offsets that are a valid index to this array.
                        let tok = unsafe { self.get_unchecked(offset) };
                        (offset + 1, Some(tok.clone()))
                } else {
                        (offset, None)
                }
        }

        fn start(&self) -> usize {
                0
        }

        fn prev(offset: usize) -> usize {
                offset.saturating_sub(1)
        }

        fn has_tokens_left(&self, offset: usize) -> Option<bool> {
                Some(offset < self.len())
        }
}

#[cfg(feature = "bytes-crate")]
impl InputType for ::bytes::Bytes {
        type Token = u8;
        type OwnedMut = ::bytes::BytesMut;

        unsafe fn next(&self, offset: usize) -> (usize, Option<Self::Token>) {
                if offset < self.len() {
                        // SAFETY: `offset < self.len()` above guarantees offset is in-bounds
                        //         We only ever return offsets that are at a character boundary
                        let tok = unsafe { self.get_unchecked(offset) };
                        (offset + 1, Some(*tok))
                } else {
                        (offset, None)
                }
        }

        fn start(&self) -> usize {
                0
        }

        fn prev(offset: usize) -> usize {
                offset.saturating_sub(1)
        }

        fn has_tokens_left(&self, offset: usize) -> Option<bool> {
                Some(offset < self.len())
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

pub struct InputOwned<I: InputType, E: ParserExtras<I> = extra::Err<I>> {
        pub(crate) input: I::Owned,
        pub(crate) cx: E::Context,
        errors: Errors<usize, E::Error>,
}

impl<I: InputType, E: ParserExtras<I>> InputOwned<I, E> {
        pub fn from_input_with_context(input: I::Owned, context: E::Context) -> Self {
                Self {
                        input,
                        cx: context,
                        errors: Errors::default(),
                }
        }
        pub fn from_input(input: I::Owned) -> Self
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
                        offset: self.input.as_ref().start(),
                        input: self.input.as_ref(),
                        cx: &self.cx,
                }
        }
        pub fn as_ref_at(&mut self, offset: usize) -> Input<'_, I, E> {
                Input {
                        offset,
                        input: self.input.as_ref(),
                        cx: &self.cx,
                }
        }
}

/// **Warning** `InputOwned` and `Input` are an unstable API.
/// This could change at any time without notice.
/// Please consider using primitives like `any` over functions in this struct. Please.
/// If you do, support is not guaranteed.
/// Changing the `offset` to arbitrary values could lead to undefined behavior. Don't modify anything in this struct if you want to be free of UB and/or segfaults.
#[derive(Debug)]
pub struct Input<'parse, I: InputType, E: ParserExtras<I> = extra::Err<I>> {
        #[doc(hidden)]
        pub offset: usize,
        #[doc(hidden)]
        pub input: &'parse I,
        // #[doc(hidden)]
        // pub errors: &'parse mut Errors<usize, E::Error>,
        // pub(crate) state: &'parse mut E::State,
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

        pub fn new_owned(input: &'parse I::Owned) -> Self {
                Self::new(input.as_ref())
        }
}

impl<'parse, I: InputType, E: ParserExtras<I>> Input<'parse, I, E> {
        /// Creates a new [`Input`] from a reference to `I` with the specified context.
        pub fn new_with_context(input: &'parse I, cx: &'parse E::Context) -> Self {
                Self {
                        offset: input.start(),
                        input,
                        cx,
                }
        }

        /// Creates a new [`Input`] from a reference to the owned container for `I` (`I::Owned`) with the specified context.
        pub fn new_owned_with_context(input: &'parse I::Owned, cx: &'parse E::Context) -> Self {
                Self::new_with_context(input.as_ref(), cx)
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
        pub(crate) fn next_inner(&mut self) -> (usize, Option<I::Token>) {
                // SAFETY: offset was generated by previous call to `Input::next`
                let (offset, token) = unsafe { self.input.next(self.offset) };
                self.offset = offset;
                (offset, token)
        }

        /// Invokes the provided `parser` on this input, returning the output or the error.
        pub fn parse<O, P: Parser<I, O, E> + ?Sized>(&mut self, parser: &P) -> Result<O, E::Error> {
                parser.parse_with(self)
        }

        /// Invokes the provided `parser` on this input, without generating an output, returning either success or the error.
        pub fn check<O, P: Parser<I, O, E> + ?Sized>(
                &mut self,
                parser: &P,
        ) -> Result<(), E::Error> {
                parser.check_with(self)
        }

        /// Save the current parse state as a [`Marker`].
        ///
        /// You can rewind back to this state later with [`Self::rewind`].
        #[inline(always)]
        pub fn save(&self) -> Marker {
                Marker {
                        offset: self.offset,
                        err_count: 0, //self.errors.secondary.len(),
                }
        }

        /// Reset the parse state to that represented by the given [`Marker`].
        ///
        /// You can create a marker with which to perform rewinding using [`Self::save`].
        /// Using a marker from another input is UB. Your parser may explode. You may get a panic. Anything could happen, really.
        #[inline(always)]
        pub fn rewind(&mut self, marker: Marker) {
                // self.errors.secondary.truncate(marker.err_count);
                self.offset = marker.offset;
        }

        /// Get the next token in the input by value. Returns [`None`] if the end of the input has been reached.
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
        pub fn span_since(&self, before: usize) -> Range<usize> {
                before..self.offset
        }

        #[inline(always)]
        pub fn next(&mut self) -> Result<I::Token, E::Error> {
                let befunge = self.offset;

                self.next_or_none()
                        .ok_or_else(|| Error::unexpected_eof(self.span_since(befunge), None))
        }

        pub fn skip(&mut self) -> Result<(), E::Error> {
                let befunge = self.offset;
                match unsafe { self.input.next(befunge) } {
                        (offset, Some(_)) => {
                                self.offset = offset;
                                Ok(())
                        }
                        (_, None) => Err(Error::unexpected_eof(self.span_since(befunge), None)),
                }
        }

        #[inline(always)]
        pub fn current(&self) -> Option<I::Token> {
                unsafe { self.input.next(I::prev(self.offset)) }.1
        }

        #[inline(always)]
        pub fn slice(&self, range: Range<usize>) -> &'parse I::Slice
        where
                I: SliceInput,
        {
                self.input.slice(range)
        }

        /// Returns the full slice of the input.
        #[inline(always)]
        pub fn full_slice(&self) -> &'parse I::Slice
        where
                I: SliceInput,
        {
                self.input.full_slice()
        }

        /// Returns a slice of the input after the offset `offset`.
        #[inline(always)]
        pub fn slice_since(&self, offset: usize) -> &'parse I::Slice
        where
                I: SliceInput,
        {
                self.input.slice(self.span_since(offset))
        }

        /// Returns a new Input with the context set to the one you provided (`cx`). You may need to specify the ParserExtras parameter manually.
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

        /// Returns a new Input with the context now being `()`. You may need to specify the ParserExtras parameter manually.
        #[inline(always)]
        pub fn no_context<E2: ParserExtras<I, Context = ()>>(&self) -> Input<'parse, I, E2> {
                Input {
                        input: self.input,
                        cx: &(),
                        offset: self.offset,
                }
        }

        /// Call this method to see if there are any tokens left.
        #[inline(always)]
        pub fn has_tokens_left(&self) -> bool {
                self.input
                        .has_tokens_left(self.offset)
                        .unwrap_or_else(|| unsafe { self.input.next(self.offset) }.1.is_some())
        }
}

#[derive(Debug, Clone, Copy)]
pub struct Marker {
        pub offset: usize,
        err_count: usize,
}

/// Implemented by inputs that represent slice-like streams of input tokens.
pub trait SliceInput: ExactSizeInput {
        /// The unsized slice type of this input. For [`str`] it's `str`, and for [`[T]`] it will be `[T]`.
        type Slice;

        /// Get the full slice of the input
        #[doc(hidden)]
        fn full_slice(&self) -> &'_ Self::Slice;

        /// Get a slice from a start and end offset
        // TODO: Make unsafe
        #[doc(hidden)]
        fn slice(&self, range: Range<usize>) -> &'_ Self::Slice;

        /// Get a slice from a start offset till the end of the input
        // TODO: Make unsafe
        #[doc(hidden)]
        fn slice_from(&self, from: RangeFrom<usize>) -> &'_ Self::Slice;
}

pub trait StrInput<C: Char>: InputType<Token = C> + SliceInput<Slice = C::Str> {}

impl ExactSizeInput for str {
        #[inline(always)]
        unsafe fn span_from(&self, range: RangeFrom<usize>) -> Range<usize> {
                range.start..self.len()
        }
}

impl<T: Clone> ExactSizeInput for [T] {
        #[inline(always)]
        unsafe fn span_from(&self, range: RangeFrom<usize>) -> Range<usize> {
                range.start..self.len()
        }
}

impl<T: Clone, const N: usize> InputType for [T; N] {
        type Token = T;
        type Owned = Self;

        unsafe fn next(&self, offset: usize) -> (usize, Option<Self::Token>) {
                (
                        offset + usize::from(offset < self.len()),
                        self.get(offset).cloned(),
                )
        }

        fn prev(offset: usize) -> usize {
                offset.saturating_sub(1)
        }

        fn start(&self) -> usize {
                0
        }

        fn has_tokens_left(&self, offset: usize) -> Option<bool> {
                Some(offset < N)
        }
}

impl<T: Clone, const N: usize> ExactSizeInput for [T; N] {
        #[inline(always)]
        unsafe fn span_from(&self, range: RangeFrom<usize>) -> Range<usize> {
                range.start..N
        }
}
impl StrInput<char> for str {}

impl SliceInput for str {
        type Slice = str;

        #[inline(always)]
        fn full_slice(&self) -> &'_ Self::Slice {
                self
        }

        #[inline(always)]
        fn slice(&self, range: Range<usize>) -> &'_ Self::Slice {
                &self[range]
        }

        #[inline(always)]
        fn slice_from(&self, from: RangeFrom<usize>) -> &'_ Self::Slice {
                &self[from]
        }
}

impl StrInput<u8> for [u8] {}

impl<T: Clone> SliceInput for [T] {
        type Slice = [T];

        #[inline(always)]
        fn full_slice(&self) -> &'_ Self::Slice {
                self
        }

        #[inline(always)]
        fn slice(&self, range: Range<usize>) -> &'_ Self::Slice {
                &self[range]
        }

        #[inline(always)]
        fn slice_from(&self, from: RangeFrom<usize>) -> &'_ Self::Slice {
                &self[from]
        }
}

impl<'a, I: InputType> InputType for &'a I {
        type Token = I::Token;
        type Owned = I::Owned;

        fn prev(offset: usize) -> usize {
                I::prev(offset)
        }

        unsafe fn next(&self, offset: usize) -> (usize, Option<Self::Token>) {
                (*self).next(offset)
        }

        fn has_tokens_left(&self, offset: usize) -> Option<bool> {
                (*self).has_tokens_left(offset)
        }

        fn start(&self) -> usize {
                (*self).start()
        }
}
