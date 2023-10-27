use std::{
        cell::Cell,
        fmt::Debug,
        ops::{Range, RangeFrom},
};

use crate::input::{ExactSizeInput, InputType};

pub struct Stream<I: Iterator> {
        tokens: Cell<(Vec<I::Item>, Option<I>)>,
}
impl<I: Iterator> Debug for Stream<I> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "")
        }
}
impl<I: Iterator> Stream<I> {
        /// Create a new stream from an [`Iterator`].
        ///
        /// # Example
        ///
        /// ```nodoc
        /// # use chumsky::{prelude::*, input::Stream};
        /// let stream = Stream::from_iter((0..10).map(|i| char::from_digit(i, 10).unwrap()));
        ///
        /// let parser = text::digits::<_, _, extra::Err<Simple<_>>>(10).collect::<String>();
        ///
        /// assert_eq!(parser.parse(stream).into_result().as_deref(), Ok("0123456789"));
        /// ```
        pub fn from_iter<J: IntoIterator<IntoIter = I>>(iter: J) -> Self {
                Self {
                        tokens: Cell::new((Vec::new(), Some(iter.into_iter()))),
                }
        }

        /// Box this stream, turning it into a [`BoxedStream`]. This can be useful in cases where your parser accepts input
        /// from several different sources and it needs to work with all of them.
        pub fn boxed<'a>(self) -> BoxedStream<'a, I::Item>
        where
                I: 'a,
        {
                let (vec, iter) = self.tokens.into_inner();
                Stream {
                        tokens: Cell::new((vec, Some(Box::new(iter.expect("no iterator?!"))))),
                }
        }

        /// Like [`Stream::boxed`], but yields an [`BoxedExactSizeStream`], which implements [`ExactSizeInput`].
        pub fn exact_size_boxed<'a>(self) -> BoxedExactSizeStream<'a, I::Item>
        where
                I: ExactSizeIterator + 'a,
        {
                let (vec, iter) = self.tokens.into_inner();
                Stream {
                        tokens: Cell::new((vec, Some(Box::new(iter.expect("no iterator?!"))))),
                }
        }
}

/// A stream containing a boxed iterator. See [`Stream::boxed`].
pub type BoxedStream<'a, T> = Stream<Box<dyn Iterator<Item = T> + 'a>>;

/// A stream containing a boxed exact-sized iterator. See [`Stream::exact_size_boxed`].
pub type BoxedExactSizeStream<'a, T> = Stream<Box<dyn ExactSizeIterator<Item = T> + 'a>>;

impl<I: Iterator> InputType for Stream<I>
where
        I::Item: Clone + std::fmt::Debug,
{
        type Token = I::Item;
        type OwnedMut = I;
        type Offset = usize;
        type Span = Range<usize>;

        fn span(&self, span: Range<Self::Offset>) -> Self::Span {
                span
        }

        #[inline(always)]
        fn start(&self) -> usize {
                0
        }

        #[inline(always)]
        fn prev(&self, offset: usize) -> usize {
                offset.saturating_sub(1)
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(skip(self)))]
        #[inline(always)]
        unsafe fn next(&self, offset: usize) -> (usize, Option<Self::Token>) {
                let mut other = Cell::new((Vec::new(), None));
                self.tokens.swap(&other);

                let (vec, iter) = other.get_mut();

                // Pull new items into the vector if we need them
                if vec.len() <= offset {
                        vec.extend(iter.as_mut().expect("no iterator?!").take(500));
                }

                // Get the token at the given offset
                let tok = vec.get(offset).map(I::Item::clone);

                self.tokens.swap(&other);

                (offset + usize::from(tok.is_some()), tok)
        }
}

impl<I: ExactSizeIterator> ExactSizeInput for Stream<I>
where
        I::Item: Clone + std::fmt::Debug,
{
        #[inline(always)]
        unsafe fn span_from(&self, range: RangeFrom<usize>) -> Range<usize> {
                let mut other = Cell::new((Vec::new(), None));
                self.tokens.swap(&other);
                let len = other.get_mut().1.as_ref().expect("no iterator?!").len();
                self.tokens.swap(&other);

                range.start..len
        }
}
