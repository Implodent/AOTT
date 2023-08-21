#![warn(clippy::pedantic)]
#![allow(clippy::inline_always)]
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use core::mem::MaybeUninit;
use core::ops::Deref;
pub mod container;
pub mod error;
pub mod input;
pub mod parser;
pub mod primitive;
pub mod stream;
pub mod text;

pub use error::IResult;

pub enum Maybe<T, R: Deref<Target = T>> {
        Ref(R),
        Val(T),
}

pub type MaybeRef<'a, T> = Maybe<T, &'a T>;

impl<T: Clone, R: Deref<Target = T>> Maybe<T, R> {
        pub fn into_clone(self) -> T {
                match self {
                        Self::Ref(r) => r.to_owned(),
                        Self::Val(v) => v,
                }
        }
}
/// A type that allows mentioning type parameters *without* all of the customary omission of auto traits that comes
/// with `PhantomData`.
struct EmptyPhantom<T>(core::marker::PhantomData<T>);

impl<T> EmptyPhantom<T> {
        const fn new() -> Self {
                Self(core::marker::PhantomData)
        }
}

impl<T> Copy for EmptyPhantom<T> {}
impl<T> Clone for EmptyPhantom<T> {
        fn clone(&self) -> Self {
                *self
        }
}
// SAFETY: This is safe because `EmptyPhantom` doesn't actually contain a `T`.
unsafe impl<T> Send for EmptyPhantom<T> {}
// SAFETY: This is safe because `EmptyPhantom` doesn't actually contain a `T`.
unsafe impl<T> Sync for EmptyPhantom<T> {}
impl<T> Unpin for EmptyPhantom<T> {}
impl<T> core::panic::UnwindSafe for EmptyPhantom<T> {}
impl<T> core::panic::RefUnwindSafe for EmptyPhantom<T> {}
// TODO: Remove this when MaybeUninit transforms to/from arrays stabilize in any form
pub(crate) trait MaybeUninitExt<T>: Sized {
        /// Identical to the unstable [`MaybeUninit::uninit_array`]
        fn uninit_array<const N: usize>() -> [Self; N];

        /// Identical to the unstable [`MaybeUninit::array_assume_init`]
        unsafe fn array_assume_init<const N: usize>(uninit: [Self; N]) -> [T; N];
}

impl<T> MaybeUninitExt<T> for MaybeUninit<T> {
        #[allow(clippy::uninit_assumed_init)]
        fn uninit_array<const N: usize>() -> [Self; N] {
                // SAFETY: Output type is entirely uninhabited - IE, it's made up entirely of `MaybeUninit`
                unsafe { MaybeUninit::uninit().assume_init() }
        }

        unsafe fn array_assume_init<const N: usize>(uninit: [Self; N]) -> [T; N] {
                let out = (&uninit as *const [Self; N] as *const [T; N]).read();
                core::mem::forget(uninit);
                out
        }
}

#[cfg(feature = "sync")]
mod sync {
        use super::*;

        pub(crate) type RefC<T> = alloc::sync::Arc<T>;
        pub(crate) type RefW<T> = alloc::sync::Weak<T>;
        pub(crate) type DynParser<'a, 'b, I, O, E> = dyn Parser<'a, I, O, E> + Send + Sync + 'b;

        /// A trait that requires either nothing or `Send` and `Sync` bounds depending on whether the `sync` feature is
        /// enabled. Used to constrain API usage succinctly and easily.
        pub trait MaybeSync: Send + Sync {}
        impl<T: Send + Sync> MaybeSync for T {}
}

#[cfg(not(feature = "sync"))]
mod sync {
        use crate::parser::Parser;

        use super::*;

        pub(crate) type RefC<T> = alloc::rc::Rc<T>;
        pub(crate) type RefW<T> = alloc::rc::Weak<T>;
        pub(crate) type DynParser<'b, I, O, E> = dyn Parser<I, O, E> + 'b;

        /// A trait that requires either nothing or `Send` and `Sync` bounds depending on whether the `sync` feature is
        /// enabled. Used to constrain API usage succinctly and easily.
        pub trait MaybeSync {}
        impl<T> MaybeSync for T {}
}

use sync::{DynParser, MaybeSync};

#[macro_export]
macro_rules! explode_extra {
        ( $O :ty ) => {
                #[inline(always)]
                fn explode_emit(&self, inp: &mut Input<I, E>) -> PResult<Emit, $O> {
                        ParserSealed::<I, $O, E>::explode::<Emit>(self, inp)
                }
                #[inline(always)]
                fn explode_check(&self, inp: &mut Input<I, E>) -> PResult<Check, $O> {
                        ParserSealed::<I, $O, E>::explode::<Check>(self, inp)
                }
        };
}
