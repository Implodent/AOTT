use crate::{
        input::{Input, InputType, StrInput},
        parser::ParserExtras,
        IResult,
};

mod private {
        pub trait Sealed {}
}

pub trait Char: Sized + Copy + PartialEq + private::Sealed + 'static {
        type Str: ?Sized + 'static;
        // type Regex;

        // Required methods
        fn from_ascii(c: u8) -> Self;
        fn is_inline_whitespace(&self) -> bool;
        fn is_whitespace(&self) -> bool;
        fn digit_zero() -> Self;
        fn is_digit(&self, radix: u32) -> bool;
        fn to_char(&self) -> char;
        /// The iterator returned by `Self::str_to_chars`.
        type StrCharIter<'a>: Iterator<Item = Self>;

        /// Turn a string of this character type into an iterator over those characters.
        fn str_to_chars(s: &Self::Str) -> Self::StrCharIter<'_>;
}

impl private::Sealed for char {}
impl Char for char {
        type Str = str;
        fn from_ascii(c: u8) -> Self {
                c as char
        }
        fn is_inline_whitespace(&self) -> bool {
                *self == ' ' || *self == '\t'
        }
        fn is_whitespace(&self) -> bool {
                char::is_whitespace(*self)
        }
        fn digit_zero() -> Self {
                '0'
        }
        fn is_digit(&self, radix: u32) -> bool {
                char::is_digit(*self, radix)
        }
        fn to_char(&self) -> char {
                *self
        }
        type StrCharIter<'a> = core::str::Chars<'a>;
        fn str_to_chars(s: &Self::Str) -> Self::StrCharIter<'_> {
                s.chars()
        }
}
impl private::Sealed for u8 {}
impl Char for u8 {
        type Str = [u8];
        fn from_ascii(c: u8) -> Self {
                c
        }
        fn is_inline_whitespace(&self) -> bool {
                *self == b' ' || *self == b'\t'
        }
        fn is_whitespace(&self) -> bool {
                self.is_ascii_whitespace()
        }
        fn digit_zero() -> Self {
                b'0'
        }
        fn is_digit(&self, radix: u32) -> bool {
                (*self as char).is_digit(radix)
        }
        fn to_char(&self) -> char {
                *self as char
        }
        type StrCharIter<'a> = core::iter::Copied<core::slice::Iter<'a, u8>>;
        fn str_to_chars(s: &Self::Str) -> Self::StrCharIter<'_> {
                s.iter().copied()
        }
}

pub mod ascii {
        use crate::{
                error::{Error, Span},
                parser::Parser,
                primitive::any,
                Maybe,
        };

        use super::*;
        /// A parser that accepts a C-style identifier.
        ///
        /// The output type of this parser is [`Char::Str`] (i.e: [`&str`] when `C` is [`char`], and [`&[u8]`] when `C` is
        /// [`u8`]).
        ///
        /// An identifier is defined as an ASCII alphabetic character or an underscore followed by any number of alphanumeric
        /// characters or underscores. The regex pattern for it is `[a-zA-Z_][a-zA-Z0-9_]*`.
        pub fn ident<'a, I: InputType + StrInput<'a, C> + 'a, C: Char, E: ParserExtras<I> + 'a>(
                inp: Input<'_, I, E>,
        ) -> IResult<'_, I, E, &'a C::Str> {
                let before = inp.offset;
                let (inp, cr) = any(inp)?;
                let chr = cr.to_char();
                let span = inp.span_since(before);
                if !(chr.is_ascii_alphabetic() || chr == '_') {
                        return Err((
                                inp,
                                Error::expected_token_found(
                                        Span::new_usize(span),
                                        vec![],
                                        crate::Maybe::Val(cr),
                                ),
                        ));
                }
                any.filter(|c: &C| c.to_char().is_ascii_alphanumeric() || c.to_char() == '_')
                        .repeated()
                        .slice()
                        .parse(inp)
        }

        /// # Panics
        /// This function panics (only in debug mode) if the `keyword` is an invalid ASCII identifier.
        #[track_caller]
        pub fn keyword<
                'a,
                C: Char + core::fmt::Debug + 'a,
                I: InputType + StrInput<'a, C> + 'a,
                E: ParserExtras<I> + 'a,
                Str: AsRef<C::Str> + 'a + Clone,
        >(
                keyword: Str,
        ) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, &'a C::Str>
        where
                C::Str: PartialEq,
        {
                #[cfg(debug_assertions)]
                {
                        let mut cs = C::str_to_chars(keyword.as_ref());
                        if let Some(c) = cs.next() {
                                assert!(c.to_char().is_ascii_alphabetic() || c.to_char() == '_', "The first character of a keyword must be ASCII alphabetic or an underscore, not {c:?}");
                        } else {
                                panic!("Keyword must have at least one character");
                        }
                        for c in cs {
                                assert!(c.to_char().is_ascii_alphanumeric() || c.to_char() == '_', "Trailing characters of a keyword must be ASCII alphanumeric or an underscore, not {c:?}");
                        }
                }
                move |input| {
                        let before = input.offset;
                        let (input, ident) = ident(input)?;
                        if ident != keyword.as_ref() {
                                let span = input.span_since(before);
                                let err = Error::expected_token_found(
                                        Span::new_usize(span),
                                        vec![],
                                        Maybe::Val(unsafe {
                                                input.input.next(before).1.unwrap_unchecked()
                                        }),
                                );
                                return Err((input, err));
                        }
                        let slice = input.input.slice(input.span_since(before));
                        Ok((input, slice))
                }
        }
}
