//! Parse using a tuple of many parsers, producing the output of the first to successfully parse.
//!
//! This primitive has a twofold improvement over a chain of [`Parser::or`] calls:
//!
//! - Rust's trait solver seems to resolve the [`Parser`] impl for this type much faster, significantly reducing
//!   compilation times.
//!
//! - Parsing is likely a little faster in some cases because the resulting parser is 'less careful' about error
//!   routing, and doesn't perform the same fine-grained error prioritization that [`Parser::or`] does.
//!
//! These qualities make this parser ideal for lexers.
//!
//! The output type of this parser is the output type of the inner parsers.
//!
//! # Examples
//!
//! ```nodoc
//! # use aott::prelude::*;
//! #[derive(Clone, Debug, PartialEq)]
//! enum Token {
//!     If,
//!     For,
//!     While,
//!     Fn,
//!     Int(u64),
//!     Ident(&'a str),
//! }
//!
//! let tokens = (
//!     text::ascii::keyword::<_, _, _, extra::Err<Simple<char>>>("if").to(Token::If),
//!     text::ascii::keyword("for").to(Token::For),
//!     text::ascii::keyword("while").to(Token::While),
//!     text::ascii::keyword("fn").to(Token::Fn),
//!     text::int(10).from_str().unwrapped().map(Token::Int),
//!     text::ascii::ident().map(Token::Ident),
//! )
//!     .padded()
//!     .repeated()
//!     .collect::<Vec<_>>();
//!
//! use Token::*;
//! assert_eq!(
//!     tokens.parse("if 56 for foo while 42 fn bar"),
//!     Ok(vec![If, Int(56), For, Ident("foo"), While, Int(42), Fn, Ident("bar")]),
//! );
//! ```
use super::*;

#[derive(Copy, Clone)]
pub struct Choice<T> {
        parsers: T,
}

pub const fn choice<T>(parsers: T) -> Choice<T> {
        Choice { parsers }
}

macro_rules! impl_choice_for_tuple {
    () => {};
    ($head:ident $($X:ident)*) => {
        impl_choice_for_tuple!($($X)*);
        impl_choice_for_tuple!(~ $head $($X)*);
    };
    (~ $Head:ident $($X:ident)+) => {
        #[allow(unused_variables, non_snake_case, unused_assignments)]
        impl<I, E, $Head, $($X),*, O> Parser<I, O, E> for Choice<($Head, $($X,)*)>
        where
            I: InputType,
            E: ParserExtras<I>,
            $Head: Parser<I, O, E>,
            $($X: Parser<I, O, E>),*
        {
            #[inline]
            fn parse_with(&self, inp: &mut Input<I, E>) -> PResult<I, O, E> {
                let mut error: E::Error;
                let before = inp.save();

                let Choice { parsers: ($Head, $($X,)*) } = self;

                match $Head.parse_with(inp) {
                    Ok(out) => return Ok(out),
                    Err(e) => { inp.rewind(before); error = e }
                }

                $(
                    match $X.parse_with(inp) {
                        Ok(out) => return Ok(out),
                        Err(e) => { inp.rewind(before); error = e }
                    }
                )*

                Err(error)
            }
            #[inline]
            fn check_with(&self, inp: &mut Input<I, E>) -> PResult<I, (), E> {
                let mut error: E::Error;
                let before = inp.save();

                let Choice { parsers: ($Head, $($X,)*) } = self;

                match $Head.check_with(inp) {
                    Ok(()) => return Ok(()),
                    Err(e) => { inp.rewind(before); error = e }
                }

                $(
                    match $X.check_with(inp) {
                        Ok(()) => return Ok(()),
                        Err(e) => { inp.rewind(before); error = e }
                    }
                )*

                Err(error)
            }

        }
    };
    (~ $Head:ident) => {
        impl<I, E, $Head, O> Parser<I, O, E> for Choice<($Head,)>
        where
            I: InputType,
            E: ParserExtras<I>,
            $Head: Parser<I, O, E>,
        {
            #[inline]
            fn parse_with(&self, inp: &mut Input<I, E>) -> PResult<I, O, E> {
                self.parsers.0.parse_with(inp)
            }

            #[inline]
            fn check_with(&self, inp: &mut Input<I, E>) -> PResult<I, (), E> {
                self.parsers.0.check_with(inp)
            }
        }
    };
}

impl_choice_for_tuple!(A_ B_ C_ D_ E_ F_ G_ H_ I_ J_ K_ L_ M_ N_ O_ P_ Q_ R_ S_ T_ U_ V_ W_ X_ Y_ Z_);
