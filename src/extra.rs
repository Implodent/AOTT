use crate::error::{Error, FundamentalError};
use crate::input::InputType;
use crate::parser::ParserExtras;
#[cfg(feature = "builtin-text")]
use crate::text::Char;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::Range;

#[derive(Default, Clone, Copy, Debug)]
pub struct Err<I: InputType, E: Error<I> = Simple<<I as InputType>::Token>>(
        PhantomData<I>,
        PhantomData<E>,
);

impl<I: InputType, E: Error<I>> ParserExtras<I> for Err<I, E> {
        type Error = E;
        type Context = ();
}

macro_rules! simple {
        ($bound:path) => {
                #[derive(Debug, Clone, derive_more::Display)]
                #[display(bound = concat!("Item: Debug + ", stringify!($bound)))]
                pub enum Simple<Item: $bound> {
                        #[display(
                                fmt = "expected end of file at {}..{}, but found {found:?}",
                                "span.start",
                                "span.end"
                        )]
                        ExpectedEOF { found: Item, span: Range<usize> },
                        #[display(
                                fmt = "unexpected end of file at {}..{}, expected {expected:?}",
                                "span.start",
                                "span.end"
                        )]
                        UnexpectedEOF {
                                span: Range<usize>,
                                expected: Option<Vec<Item>>,
                        },
                        #[display(
                                fmt = "expected {expected:?} at {}..{}, but found {found:?}",
                                "span.start",
                                "span.end"
                        )]
                        ExpectedTokenFound {
                                span: Range<usize>,
                                expected: Vec<Item>,
                                found: Item,
                        },
                        #[cfg(feature = "builtin-text")]
                        #[display(
                                fmt = "{error} at {}..{}, last token was {last_token:?}",
                                "span.start",
                                "span.end"
                        )]
                        Text {
                                span: Range<usize>,
                                error: crate::text::CharLabel<Item>,
                                last_token: Option<Item>,
                        },
                        #[display(
                                fmt = "{label} at {}..{}, last token was {last_token:?}",
                                "span.start",
                                "span.end"
                        )]
                        Builtin {
                                span: Range<usize>,
                                label: crate::error::BuiltinLabel,
                                last_token: Option<Item>,
                        },
                }

                impl<Item: $bound, I: InputType<Token = Item>> FundamentalError<I>
                        for Simple<Item>
                {
                        fn expected_eof_found(span: Range<usize>, found: Item) -> Self {
                                Self::ExpectedEOF { found, span }
                        }
                        fn expected_token_found(
                                span: Range<usize>,
                                expected: Vec<<I as InputType>::Token>,
                                found: <I as InputType>::Token,
                        ) -> Self {
                                Self::ExpectedTokenFound {
                                        span,
                                        expected,
                                        found,
                                }
                        }
                        fn unexpected_eof(
                                span: Range<usize>,
                                expected: Option<Vec<<I as InputType>::Token>>,
                        ) -> Self {
                                Self::UnexpectedEOF { span, expected }
                        }
                }

                impl<Item: $bound, I: InputType<Token = Item>>
                        crate::error::LabelError<I, crate::error::BuiltinLabel> for Simple<Item>
                {
                        fn from_label(
                                span: Range<usize>,
                                label: crate::error::BuiltinLabel,
                                last_token: Option<Item>,
                        ) -> Self {
                                Self::Builtin {
                                        span,
                                        label,
                                        last_token,
                                }
                        }
                }

                #[cfg(feature = "builtin-text")]
                impl<C: Char, I: InputType<Token = C>>
                        crate::error::LabelError<I, crate::text::CharLabel<C>> for Simple<C>
                {
                        fn from_label(
                                span: Range<usize>,
                                error: crate::text::CharLabel<C>,
                                last_token: Option<C>,
                        ) -> Self {
                                Self::Text {
                                        span,
                                        error,
                                        last_token,
                                }
                        }
                }

                #[cfg(feature = "std")]
                impl<Item: Debug + $bound + 'static> std::error::Error for Simple<Item> {
                        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                                None
                        }
                }
                #[cfg(all(feature = "nightly", not(feature = "std")))]
                impl<Item: Debug + $bound + 'static> core::error::Error for Simple<Item> {
                        fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
                                None
                        }
                }
        };
}

#[cfg(feature = "builtin-text")]
simple!(Char);

#[cfg(not(feature = "builtin-text"))]
simple!(core::fmt::Debug);
