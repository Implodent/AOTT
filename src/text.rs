use core::{borrow::Borrow, marker::PhantomData};

use crate::{
        container::OrderedSeq,
        derive::parser,
        error::{Error, Span},
        input::{Input, InputType, StrInput},
        parser::ParserExtras,
        prelude::Parser,
        primitive::*,
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
                                crate::MaybeDeref::Val(cr),
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
                                crate::MaybeDeref::Val(unsafe {
                                        input.input.next(before).1.unwrap_unchecked()
                                }),
                        );
                        return Err((input, err));
                }
                let slice = input.input.slice(input.span_since(before));
                Ok((input, slice))
        }
}

static NEWLINE_CHARACTERS_AFTER_CRLF: [char; 6] = [
        '\r',       // Carriage return
        '\x0B',     // Vertical tab
        '\x0C',     // Form feed
        '\u{0085}', // Next line
        '\u{2028}', // Line separator
        '\u{2029}', // Paragraph separator
];

#[parser(extras = E)]
/// A parser that accepts (and ignores) any newline characters or character sequences.
///
/// The output type of this parser is `()`.
///
/// This parser is quite extensive, recognizing:
///
/// - Line feed (`\n`)
/// - Carriage return (`\r`)
/// - Carriage return + line feed (`\r\n`)
/// - Vertical tab (`\x0B`)
/// - Form feed (`\x0C`)
/// - Next line (`\u{0085}`)
/// - Line separator (`\u{2028}`)
/// - Paragraph separator (`\u{2029}`)
///
/// # Examples
///
/// ```
/// # use aott::prelude::*;
/// let newline = text::newline::<_, SimpleExtras<char>>;
///
/// assert_eq!(newline.parse_from(&"\n").unwrap().1, ());
/// assert_eq!(newline.parse_from(&"\r").unwrap().1, ());
/// assert_eq!(newline.parse_from(&"\r\n").unwrap().1, ());
/// assert_eq!(newline.parse_from(&"\x0B").unwrap().1, ());
/// assert_eq!(newline.parse_from(&"\x0C").unwrap().1, ());
/// assert_eq!(newline.parse_from(&"\u{0085}").unwrap().1, ());
/// assert_eq!(newline.parse_from(&"\u{2028}").unwrap().1, ());
/// assert_eq!(newline.parse_from(&"\u{2029}").unwrap().1, ());
/// ```
pub fn newline<I: InputType, E: ParserExtras<I>>(input: I)
where
        I::Token: Char + PartialEq,
{
        // parses \r, which is either the OSX newline, or the start of a Windows newline (\r\n)
        maybe(cr)
                .ignore_then(lf) // parses \n, which is either a Linux newline, or the end of a Windows newline (\r\n)
                .or(filter(|cr: &I::Token| {
                        NEWLINE_CHARACTERS_AFTER_CRLF.contains(&cr.to_char())
                }))
                .ignored()
}

#[parser(extras = E)]
/// Parses a unix-style newline. (\n)
pub fn lf<I: InputType, E: ParserExtras<I>>(input: I) -> I::Token
where
        I::Token: Char + PartialEq,
{
        just(Char::from_ascii(b'\n'))(input)
}

#[parser(extras = E)]
/// Parses a DOS(Windows)-style newline. (\r\n)
pub fn crlf<I: InputType, E: ParserExtras<I>>(input: I) -> [I::Token; 2]
where
        I::Token: Char + PartialEq,
{
        just([Char::from_ascii(b'\r'), Char::from_ascii(b'\n')])(input)
}

#[parser(extras = E)]
/// Parses an OSX(MacOS)-style newline. (\r)
pub fn cr<I: InputType, E: ParserExtras<I>>(input: I) -> I::Token
where
        I::Token: Char + PartialEq,
{
        just(Char::from_ascii(b'\r'))(input)
}

/// Parses a sequence of characters, ignoring the character's case.
pub fn just_ignore_case<
        'a,
        'parse,
        I: InputType,
        E: ParserExtras<I>,
        T: OrderedSeq<'a, I::Token> + Clone,
>(
        seq: T,
) -> impl Fn(Input<'parse, I, E>) -> IResult<'parse, I, E, T>
where
        I::Token: Char + PartialEq + Clone + 'static,
{
        move |mut input| {
                if let Some(err) = seq.seq_iter().find_map(|next| {
                        let befunge = input.offset;
                        let next = T::to_maybe_ref(next);
                        match input.next_inner() {
                                (_, Some(token))
                                        if next.borrow_as_t().to_char().eq_ignore_ascii_case(
                                                &token.borrow().to_char(),
                                        ) =>
                                {
                                        None
                                }
                                (_, found) => Some(Error::expected_token_found_or_eof(
                                        Span::new_usize(input.span_since(befunge)),
                                        vec![next.into_clone()],
                                        found.map(crate::MaybeDeref::Val),
                                )),
                        }
                }) {
                        Err((input, err))
                } else {
                        Ok((input, seq.clone()))
                }
        }
}

/// Parses a non-negative integer in the specified radix.
///
/// An integer is defined as a non-empty sequence of ASCII digits, where the first digit is non-zero or the sequence
/// has length one.
///
/// The output type of this parser is `I::Slice` (i.e: [`&str`] when `I` is [`&str`], and [`&[u8]`]
/// when `I` is [`&[u8]`]).
///
/// The `radix` parameter functions identically to [`char::is_digit`]. If in doubt, choose `10`.
pub fn int<'parse, 'a, I: InputType + StrInput<'a, C>, C: Char, E: ParserExtras<I>>(
        radix: u32,
) -> impl Fn(Input<'parse, I, E>) -> IResult<'parse, I, E, &'a C::Str> {
        move |input| {
                with_slice(input, move |input| {
                        let (input, cr) = any(input)?;
                        let befunge = input.offset;
                        if !(cr.is_digit(radix) && cr != C::digit_zero()) {
                                let err = Error::expected_token_found(
                                        Span::new_usize(input.span_since(befunge)),
                                        vec![],
                                        crate::MaybeDeref::Val(cr),
                                );
                                return Err((input, err));
                        }
                        // hehe
                        any.filter(move |cr: &C| cr.is_digit(radix))
                                .repeated()
                                .ignored()
                                .or(just(C::digit_zero()).ignored())
                                .check(input)
                })
        }
}

#[derive(Copy, Clone)]
pub struct Padded<A, C>(A, PhantomData<C>);

/// A parser that accepts and ignores any number of whitespace characters before or after another parser.
pub fn padded<
        'a,
        I: InputType + StrInput<'a, C>,
        E: ParserExtras<I>,
        C: Char,
        O,
        A: Parser<I, O, E>,
>(
        parser: A,
) -> Padded<A, C> {
        Padded(parser, PhantomData)
}

impl<
                'a,
                I: InputType + StrInput<'a, C>,
                E: ParserExtras<I>,
                C: Char,
                O,
                A: Parser<I, O, E>,
        > Parser<I, O, E> for Padded<A, C>
{
        fn check<'parse>(&self, mut input: Input<'parse, I, E>) -> IResult<'parse, I, E, ()> {
                input.skip_while(Char::is_whitespace);
                let (mut input, ()) = self.0.check(input)?;
                input.skip_while(Char::is_whitespace);
                Ok((input, ()))
        }

        fn parse<'parse>(&self, mut input: Input<'parse, I, E>) -> IResult<'parse, I, E, O> {
                input.skip_while(Char::is_whitespace);
                let (mut input, output) = self.0.parse(input)?;
                input.skip_while(Char::is_whitespace);
                Ok((input, output))
        }
}
