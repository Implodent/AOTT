#![allow(dead_code)]
use core::{
        hash::Hash,
        ops::{Range, RangeFrom},
};

use crate::{
        error::Located,
        parser::{Emit, Parser, ParserExtras, SimpleExtras},
        stream::Stream,
};

use self::private::Sealed;

mod private {
        pub trait Sealed {}
}

#[allow(clippy::module_name_repetitions)]
pub trait InputType: Sealed {
        #[doc(hidden)]
        type Offset: Copy + Hash + Ord + Into<usize>;
        type Token;

        #[doc(hidden)]
        fn start(&self) -> Self::Offset;

        #[doc(hidden)]
        // # Safety
        // If `offset` is not strictly the one provided by `Self::start` or returned as the first tuple value from this function,
        // calling `next` is undefined behavior. It may index memory outside of the desired range, it may segfault, it may panic etc. etc.
        // Stay safe and don't use this api unless you want to explode.
        unsafe fn next(&self, offset: Self::Offset) -> (Self::Offset, Option<Self::Token>);

        #[doc(hidden)]
        fn prev(offset: Self::Offset) -> Self::Offset;
}

impl<'a> Sealed for &'a str {}
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

impl<'a, T> Sealed for &'a [T] {}
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

impl<I: Iterator> Sealed for Stream<I> {}

#[doc(hidden)]
pub trait ExactSizeInput: InputType {
        unsafe fn span_from(&self, range: RangeFrom<Self::Offset>) -> Range<usize>;
}

#[doc(hidden)]
#[derive(Debug)]
pub struct Errors<T, E> {
        pub alt: Option<Located<T, E>>,
        pub(crate) secondary: Vec<Located<T, E>>,
}

impl<T, E> Default for Errors<T, E> {
        fn default() -> Self {
                Self {
                        alt: None,
                        secondary: vec![],
                }
        }
}

#[doc(hidden)]
pub struct InputOwned<I: InputType, E: ParserExtras<I>> {
        pub(crate) input: I,
        pub(crate) cx: E::Context,
        pub errors: Errors<I::Offset, E::Error>,
}

impl<I: InputType, E: ParserExtras<I>> InputOwned<I, E> {
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
        pub fn as_ref_at_zero<'this>(&'this mut self) -> Input<'this, I, E> {
                Input {
                        offset: self.input.start(),
                        input: &self.input,
                        errors: &mut self.errors,
                        cx: &self.cx,
                }
        }
        pub fn as_ref_at<'this>(&'this mut self, offset: I::Offset) -> Input<'this, I, E> {
                Input {
                        offset,
                        input: &self.input,
                        errors: &mut self.errors,
                        cx: &self.cx,
                }
        }
}

// InputRef
#[derive(Debug)]
pub struct Input<'parse, I: InputType, E: ParserExtras<I> = SimpleExtras<I>> {
        #[doc(hidden)]
        pub offset: I::Offset,
        #[doc(hidden)]
        pub(crate) input: &'parse I,
        #[doc(hidden)]
        pub errors: &'parse mut Errors<I::Offset, E::Error>,
        // pub(crate) state: &'parse mut E::State,
        #[doc(hidden)]
        pub(crate) cx: &'parse E::Context,
}

impl<'parse, I: InputType, E: ParserExtras<I>> Input<'parse, I, E> {
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
        pub fn parse_old<O, P: Parser<I, O, E>>(self, parser: &P) -> (Self, Result<O, E::Error>) {
                let (this, result) = parser.explode::<Emit>(self);
                if let Ok(ok) = result {
                        (this, Ok(ok))
                } else {
                        let error = this
                                .errors
                                .alt
                                .take()
                                .expect("returned Err(()) but no alt error. bad parser!")
                                .err;
                        (this, Err(error))
                }
        }
        /// # Panics
        /// A parser, if it returns an error, must put the error into input.errors.alt.
        /// If a parser returns Err(()), but there is no error in .errors.alt, a panic happens.
        pub fn parse<O, P: Parser<I, O, E>>(
                self,
                parser: &P,
        ) -> Result<(Self, O), (Self, E::Error)> {
                let (this, result) = self.parse_old(parser);
                match result {
                        Ok(ok) => Ok((this, ok)),
                        Err(err) => Err((this, err)),
                }
        }
        /// Save the current parse state as a [`Marker`].
        ///
        /// You can rewind back to this state later with [`InputRef::rewind`].
        #[inline(always)]
        pub fn save(&self) -> Marker<I> {
                Marker {
                        offset: self.offset,
                        err_count: self.errors.secondary.len(),
                }
        }

        /// Reset the parse state to that represented by the given [`Marker`].
        ///
        /// You can create a marker with which to perform rewinding using [`InputRef::save`].
        /// Using a marker from another input is UB. Your parser may explode. You may get a panic.
        #[inline(always)]
        pub fn rewind(&mut self, marker: Marker<I>) {
                self.errors.secondary.truncate(marker.err_count);
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
}

#[derive(Debug)]
pub struct Marker<I: InputType> {
        offset: I::Offset,
        err_count: usize,
}
impl<I: InputType> Clone for Marker<I> {
        fn clone(&self) -> Self {
                *self
        }
}
impl<I: InputType> Copy for Marker<I> {}
