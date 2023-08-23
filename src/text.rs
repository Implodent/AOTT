use crate::{
        container::OrderedSeq,
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
}

pub fn keyword<
        'a,
        C: Char + 'a,
        I: InputType + StrInput<'a, C>,
        E: ParserExtras<I>,
        Str: AsRef<C::Str> + 'a + Clone,
>(
        keyword: Str,
) -> impl Fn(Input<'_, I, E>) -> IResult<'_, I, E, &'a C::Str>
where
        C::Str: PartialEq,
{
        move |input| todo!()
}
