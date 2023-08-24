macro_rules! define_number_mod {
    ($v:vis $($num:ident),*) => {
            $v mod big {
                $(define_number_mod!(~ from_be_bytes $num);)*
            }
            $v mod native {
                $(define_number_mod!(~ from_ne_bytes $num);)*
            }
            $v mod little {
                $(define_number_mod!(~ from_le_bytes $num);)*
            }
        };
        (~ $from_endian_name:ident $num:ident) => {
                #[doc = concat!("# Errors\nThis parser returns an error if there is not enough bytes to parse a full ", stringify!($num), ".\nThere are no other causes for errors.")]
                pub fn $num<
                        'parse,
                        'a,
                        I: $crate::input::InputType<Token = u8> + $crate::input::SliceInput<'a>,
                        E: $crate::parser::ParserExtras<I>,
                >(
                        input: $crate::input::Input<'parse, I, E>,
                ) -> $crate::error::IResult<'parse, I, E, $num> {
                    use $crate::parser::Parser;
                    let (input, bytes) = $crate::primitive::take_exact::<{core::mem::size_of::<$num>()}>().parse(input)?;
                    Ok((input, $num::$from_endian_name(bytes)))
                }
        };
}

define_number_mod![pub u8, u16, u32, u64, u128, i8, i16, i32, i64, i128];
