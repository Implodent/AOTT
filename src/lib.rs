#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::inline_always)]
#![allow(clippy::wildcard_imports)]
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use core::mem::MaybeUninit;
use core::ops::Deref;
#[cfg(feature = "builtin-bytes")]
pub mod bytes;
pub mod container;
pub mod error;
pub mod input;
pub mod parser;
pub mod primitive;
#[cfg(feature = "error-recovery")]
pub mod recovery;
pub mod stream;
pub use aott_derive as derive;

#[cfg(feature = "builtin-text")]
pub mod text;

pub mod prelude {
        pub use crate::derive::parser;
        pub use crate::error::{Error, IResult, Simple, Span};
        pub use crate::input::{
                ExactSizeInput, Input, InputOwned, InputType, SliceInput, StrInput,
        };
        pub use crate::parser::{Parser, ParserExtras, SimpleExtras};
        pub use crate::primitive::*;
        pub use crate::stream::Stream;
}

pub use error::IResult;

pub enum Maybe<T, R: Deref<Target = T>> {
        Ref(R),
        Val(T),
}

pub type MaybeRef<'a, T> = Maybe<T, &'a T>;

impl<'a, T> MaybeRef<'a, T> {
        pub fn borrow_as_t<'b: 'a>(&'b self) -> &'b T {
                match self {
                        Self::Ref(r) => r,
                        Self::Val(ref v) => v,
                }
        }
}

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
                let out = (std::ptr::addr_of!(uninit) as *const [T; N]).read();
                core::mem::forget(uninit);
                out
        }
}

#[cfg(feature = "sync")]
#[allow(dead_code)]
#[doc(hidden)]
pub mod sync {
        use super::*;

        pub(crate) type RefC<T> = alloc::sync::Arc<T>;
        pub(crate) type RefW<T> = alloc::sync::Weak<T>;
        pub(crate) type DynParser<'a, 'b, I, O, E> = dyn Parser<'a, I, O, E> + Send + Sync + 'b;
        pub type SyncArray<T> = alloc::sync::Arc<[T]>;

        /// A trait that requires either nothing or `Send` and `Sync` bounds depending on whether the `sync` feature is
        /// enabled. Used to constrain API usage succinctly and easily.
        pub trait MaybeSync: Send + Sync {}
        impl<T: Send + Sync> MaybeSync for T {}
}

#[cfg(not(feature = "sync"))]
#[doc(hidden)]
#[allow(dead_code)]
pub mod sync {
        use crate::parser::Parser;

        use super::alloc;

        pub type RefC<T> = alloc::rc::Rc<T>;
        pub type RefW<T> = alloc::rc::Weak<T>;
        pub type DynParser<'b, I, O, E> = dyn Parser<I, O, E> + 'b;
        pub type SyncArray<T> = alloc::rc::Rc<[T]>;

        /// A trait that requires either nothing or `Send` and `Sync` bounds depending on whether the `sync` feature is
        /// enabled. Used to constrain API usage succinctly and easily.
        pub trait MaybeSync {}
        impl<T> MaybeSync for T {}
}

use sync::{DynParser, MaybeSync};
