use super::*;

/// If `I` is `true`, outputs of `A` and `B` are collected into a tuple of `(a_out, b_out)`.
/// If `I` is `false`, and if `AI` is false, `B` is ran in check mode (no outputs produced), and only the output of `A` is returned; if `AI` is `true`, `A` is ignored; parsers are ran in order of `A` then `B`.
#[derive(Copy, Clone)]
pub struct Then<O1, O2, A, B, const I: bool, const AI: bool>(
        pub(crate) A,
        pub(crate) B,
        pub(crate) core::marker::PhantomData<(O1, O2)>,
);

impl<I: InputType, E: ParserExtras<I>, O1, O2, A: Parser<I, O1, E>, B: Parser<I, O2, E>>
        Parser<I, (O1, O2), E> for Then<O1, O2, A, B, true, false>
{
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, (O1, O2), E> {
                Ok((self.0.parse_with(input)?, self.1.parse_with(input)?))
        }
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.0.check_with(input)?;
                self.1.check_with(input)
        }
}
impl<I: InputType, E: ParserExtras<I>, O1, O2, A: Parser<I, O1, E>, B: Parser<I, O2, E>>
        Parser<I, O1, E> for Then<O1, O2, A, B, false, false>
{
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O1, E> {
                let a = self.0.parse_with(input)?;
                self.1.check_with(input)?;
                Ok(a)
        }
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.0.check_with(input)?;
                self.1.check_with(input)?;
                Ok(())
        }
}
impl<I: InputType, E: ParserExtras<I>, O1, O2, A: Parser<I, O1, E>, B: Parser<I, O2, E>>
        Parser<I, O2, E> for Then<O1, O2, A, B, false, true>
{
        fn parse_with(&self, input: &mut Input<I, E>) -> PResult<I, O2, E> {
                self.0.check_with(input)?;
                self.1.parse_with(input)
        }
        fn check_with(&self, input: &mut Input<I, E>) -> PResult<I, (), E> {
                self.0.check_with(input)?;
                self.1.check_with(input)
        }
}

/// Parse using a tuple of many parsers, producing a tuple of outputs if all successfully parse,
/// otherwise returning an error if any parsers fail.
///
/// This parser is to [`Parser::then`] as [`choice()`] is to [`Parser::or`]
#[deprecated(since = "0.3.0", note = "just use the tuple")]
pub const fn tuple<T>(parsers: T) -> T {
        parsers
}

// impl<I, O, E, P, const N: usize> Parser<I, [O; N], E> for Tuple<[P; N]>
// where
//         I: InputType,
//         E: ParserExtras<I>,
//         P: Parser<I, O, E>,
// {
//         #[inline]
//         fn explode<'parse, M: Mode>(
//                 &self,
//                 inp: Input<'parse, I, E>,
//         ) -> PResult<'parse, I, E, M, [O; N]> {
//                 let mut arr: [MaybeUninit<_>; N] = MaybeUninitExt::uninit_array();
//             let mut i = inp;
//                 for (p, res) in self.parsers
//                         .iter()
//                         .zip(arr.iter_mut()) {
//                             let (input, out) = match p.explode::<M>(inp) {
//                                 (input)
//                             };

//                         }
//                         // .try_for_each(|(p, res)| {
//                         //         res.write(p.explode::<M>(inp)?);
//                         //         Ok(())
//                         // })?;
//                 // SAFETY: We guarantee that all parers succeeded and as such all items have been initialized
//                 //         if we reach this point
//                 Ok(M::array(unsafe { MaybeUninitExt::array_assume_init(arr) }))
//         }

//         explode_extra!([O; N]);
// }

macro_rules! flatten_map {
    // map a single element into a 1-tuple
    (<$M:ident> $head:ident) => {
        $M::map(
            $head,
            |$head| ($head,),
        )
    };
    // combine two elements into a 2-tuple
    (<$M:ident> $head1:ident $head2:ident) => {
        $M::combine(
            $head1,
            $head2,
            |$head1, $head2| ($head1, $head2),
        )
    };
    // combine and flatten n-tuples from recursion
    (<$M:ident> $head:ident $($X:ident)+) => {
        $M::combine(
            $head,
            flatten_map!(
                <$M>
                $($X)+
            ),
            |$head, ($($X),+)| ($head, $($X),+),
        )
    };
}

macro_rules! impl_tuple_for_tuple {
    () => {};
    ($head:ident $ohead:ident $($X:ident $O:ident)*) => {
        impl_tuple_for_tuple!($($X $O)*);
        impl_tuple_for_tuple!(~ $head $ohead $($X $O)*);
    };
    (~ $($X:ident $O:ident)*) => {
        #[allow(unused_variables, non_snake_case)]
        impl<I, E, $($X),*, $($O),*> Parser<I, ($($O,)*), E> for ($($X,)*)
        where
            I: InputType,
            E: ParserExtras<I>,
            $($X: Parser<I, $O, E>),*
        {
            #[inline]
            fn parse_with(&self, inp: &mut Input<I, E>) -> PResult<I, ($($O,)*), E> {
                let ($($X,)*) = self;

                $(
                    let $X = $X.parse_with(inp)?;
                )*

                Ok(flatten_map!(<Emit> $($X)*))
            }

            #[inline]
            fn check_with(&self, inp: &mut Input<I, E>) -> PResult<I, (), E> {
                let ($($X,)*) = self;

                $(
                    $X.check_with(inp)?;
                )*

                Ok(())
            }
        }
    };
}

impl_tuple_for_tuple! {
    A_ OA
    B_ OB
    C_ OC
    D_ OD
    E_ OE
    F_ OF
    G_ OG
    H_ OH
    I_ OI
    J_ OJ
    K_ OK
    L_ OL
    M_ OM
    N_ ON
    O_ OO
    P_ OP
    Q_ OQ
    R_ OR
    S_ OS
    T_ OT
    U_ OU
    V_ OV
    W_ OW
    X_ OX
    Y_ OY
    Z_ OZ
}
