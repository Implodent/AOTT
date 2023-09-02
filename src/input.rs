#![allow(dead_code)]
use crate::{
        error::{Error, Located, Span},
        extra,
        parser::{Parser, ParserExtras},
        text::Char,
};
use core::{
        hash::Hash,
        ops::{Range, RangeFrom},
};
use num_traits::{One, Zero};

#[allow(clippy::module_name_repetitions)]
pub trait InputType {
        #[doc(hidden)]
        type Offset: Copy + Hash + Ord + Into<usize> + Zero + One;
        type Token;

        #[doc(hidden)]
        fn start(&self) -> Self::Offset;

        /// # Safety
        /// If `offset` is not strictly the one provided by `Self::start` or returned as the first tuple value from this function,
        /// calling `next` is undefined behavior. It may index memory outside of the desired range, it may segfault, it may panic etc. etc.
        /// Stay safe and don't use this api unless you want to explode.
        #[doc(hidden)]
        unsafe fn next(&self, offset: Self::Offset) -> (Self::Offset, Option<Self::Token>);

        #[doc(hidden)]
        fn prev(offset: Self::Offset) -> Self::Offset;
}

impl<'a> InputType for &'a str {
        type Token = char;
        type Offset = usize;

        #[inline]
        fn start(&self) -> Self::Offset {
                0
        }

        fn prev(offset: Self::Offset) -> Self::Offset {
                offset.saturating_sub(1)
        }

        #[inline(always)]
        unsafe fn next(&self, offset: Self::Offset) -> (Self::Offset, Option<Self::Token>) {
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
        type Offset = usize;
        type Token = T;

        unsafe fn next(&self, offset: Self::Offset) -> (Self::Offset, Option<Self::Token>) {
                if offset < self.len() {
                        // SAFETY: `offset < self.len()` above guarantees offset is in-bounds
                        //         We only ever return offsets that are at a character boundary
                        let tok = unsafe { self.get_unchecked(offset) };
                        (offset + 1, Some(tok.clone()))
                } else {
                        (offset, None)
                }
        }

        fn start(&self) -> Self::Offset {
                0
        }

        fn prev(offset: Self::Offset) -> Self::Offset {
                offset.saturating_sub(1)
        }
}

#[doc(hidden)]
pub trait ExactSizeInput: InputType {
        unsafe fn span_from(&self, range: RangeFrom<Self::Offset>) -> Range<usize>;
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
        pub(crate) input: I,
        pub(crate) cx: E::Context,
        errors: Errors<I::Offset, E::Error>,
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
        pub fn as_ref_at(&mut self, offset: I::Offset) -> Input<'_, I, E> {
                Input {
                        offset,
                        input: &self.input,
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
        pub offset: I::Offset,
        #[doc(hidden)]
        pub input: &'parse I,
        // #[doc(hidden)]
        // pub errors: &'parse mut Errors<I::Offset, E::Error>,
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
}

impl<'parse, I: InputType, E: ParserExtras<I>> Input<'parse, I, E> {
        pub fn new_with_context(input: &'parse I, cx: &'parse E::Context) -> Self {
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

        #[inline]
        pub(crate) fn skip_while(&mut self, mut f: impl FnMut(&I::Token) -> bool) {
                loop {
                        // SAFETY: offset was generated by previous call to `Input::next`
                        let (offset, token) = unsafe { self.input.next(self.offset) };
                        if token.filter(&mut f).is_none() {
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
                (self.offset, token)
        }

        /// # Panics
        /// A parser, if it returns an error, must put the error into input.errors.alt.
        /// If a parser returns Err(()), but there is no error in .errors.alt, a panic happens.
        pub fn parse<O, P: Parser<I, O, E> + ?Sized>(
                self,
                parser: &P,
        ) -> Result<(Self, O), (Self, E::Error)> {
                parser.parse(self)
        }

        pub fn check<O, P: Parser<I, O, E>>(
                self,
                parser: &P,
        ) -> Result<(Self, ()), (Self, E::Error)> {
                parser.check(self)
        }
        /// Save the current parse state as a [`Marker`].
        ///
        /// You can rewind back to this state later with [`InputRef::rewind`].
        #[inline(always)]
        pub fn save(&self) -> Marker<I> {
                Marker {
                        offset: self.offset,
                        err_count: 0, //self.errors.secondary.len(),
                }
        }

        /// Reset the parse state to that represented by the given [`Marker`].
        ///
        /// You can create a marker with which to perform rewinding using [`InputRef::save`].
        /// Using a marker from another input is UB. Your parser may explode. You may get a panic.
        #[inline(always)]
        pub fn rewind(&mut self, marker: Marker<I>) {
                // self.errors.secondary.truncate(marker.err_count);
                self.offset = marker.offset;
        }

        /// Get the next token in the input by value. Returns `None` if the end of the input has been reached.
        #[inline(always)]
        pub fn next(&mut self) -> Option<I::Token> {
                self.next_inner().1
        }
        /// Peek the next token in the input. Returns `None` if the end of the input has been reached.
        #[inline(always)]
        pub fn peek(&self) -> Option<I::Token> {
                // SAFETY: offset was generated by previous call to `Input::next`
                unsafe { self.input.next(self.offset).1 }
        }
        #[inline(always)]
        pub fn span_since(&self, before: I::Offset) -> Range<I::Offset> {
                before..self.offset
        }
        #[inline(always)]
        pub fn next_or_eof(&mut self) -> Result<I::Token, E::Error> {
                let befunge = self.offset;
                match self.next_inner() {
                        (_, Some(token)) => Ok(token),
                        (_, None) => Err(Error::unexpected_eof(
                                Span::new_usize(self.span_since(befunge)),
                                None,
                        )),
                }
        }
}

#[derive(Debug)]
pub struct Marker<I: InputType> {
        pub offset: I::Offset,
        err_count: usize,
}
impl<I: InputType> Clone for Marker<I> {
        fn clone(&self) -> Self {
                *self
        }
}
impl<I: InputType> Copy for Marker<I> {}

/// Implemented by inputs that represent slice-like streams of input tokens.
pub trait SliceInput<'a>: ExactSizeInput {
        /// The unsized slice type of this input. For [`&str`] it's `&'a str`, and for [`&[T]`] it will be `&'a [T]`.
        type Slice: 'a;

        /// Get the full slice of the input
        #[doc(hidden)]
        fn full_slice(&self) -> Self::Slice;

        /// Get a slice from a start and end offset
        // TODO: Make unsafe
        #[doc(hidden)]
        fn slice(&self, range: Range<Self::Offset>) -> Self::Slice;

        /// Get a slice from a start offset till the end of the input
        // TODO: Make unsafe
        #[doc(hidden)]
        fn slice_from(&self, from: RangeFrom<Self::Offset>) -> Self::Slice;
}

pub trait StrInput<'a, C: Char>:
        InputType<Offset = usize, Token = C> + SliceInput<'a, Slice = &'a C::Str>
{
}
impl<'a> ExactSizeInput for &'a str {
        #[inline(always)]
        unsafe fn span_from(&self, range: RangeFrom<Self::Offset>) -> Range<Self::Offset> {
                range.start..self.len()
        }
}
impl<'a, T: Clone> ExactSizeInput for &'a [T] {
        #[inline(always)]
        unsafe fn span_from(&self, range: RangeFrom<Self::Offset>) -> Range<Self::Offset> {
                range.start..self.len()
        }
}
// impl<'a, T: Clone + 'a, const N: usize> ExactSizeInput for &'a [T; N] {
//         #[inline(always)]
//         unsafe fn span_from(&self, range: RangeFrom<Self::Offset>) -> Range<Self::Offset> {
//                 (range.start..N).into()
//         }
// }
impl<'a> StrInput<'a, char> for &'a str {}

impl<'a> SliceInput<'a> for &'a str {
        type Slice = &'a str;

        #[inline(always)]
        fn full_slice(&self) -> Self::Slice {
                *self
        }

        #[inline(always)]
        fn slice(&self, range: Range<Self::Offset>) -> Self::Slice {
                &self[range]
        }

        #[inline(always)]
        fn slice_from(&self, from: RangeFrom<Self::Offset>) -> Self::Slice {
                &self[from]
        }
}

impl<'a> StrInput<'a, u8> for &'a [u8] {}

impl<'a, T: Clone> SliceInput<'a> for &'a [T] {
        type Slice = &'a [T];

        #[inline(always)]
        fn full_slice(&self) -> Self::Slice {
                *self
        }

        #[inline(always)]
        fn slice(&self, range: Range<Self::Offset>) -> Self::Slice {
                &self[range]
        }

        #[inline(always)]
        fn slice_from(&self, from: RangeFrom<Self::Offset>) -> Self::Slice {
                &self[from]
        }
}
