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
                #[doc = concat!("Parses a nunebr of type ", stringify!($num), ".\n# Errors\nThis parser returns an error if there is not enough bytes to parse a full ", stringify!($num), ".\nThere are no other causes for errors.")]
                pub fn $num<
                        I: $crate::input::InputType<Token = u8>,
                        E: $crate::parser::ParserExtras<I>,
                >(
                        input: &mut $crate::input::Input<I, E>,
                ) -> $crate::error::PResult<I, $num, E> {
                    use $crate::parser::Parser;
                    let bytes = $crate::primitive::take_exact::<{std::mem::size_of::<$num>()}>().parse_with(input)?;
                    Ok($num::$from_endian_name(bytes))
                }
        };
}

define_number_mod![pub u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64];
