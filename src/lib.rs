#![doc = include_str!("../README_docsrs.md")]
#![warn(clippy::pedantic)]
#![allow(
        clippy::module_name_repetitions,
        clippy::wildcard_imports,
        clippy::inline_always,
        rustdoc::private_intra_doc_links
)]
#![cfg_attr(feature = "nightly", feature(associated_type_defaults))]
#![cfg_attr(all(feature = "nightly", not(feature = "std")), feature(error_in_core))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
// allow derive to use ::aott::PResult
extern crate self as aott;

use core::mem::MaybeUninit;
use core::ops::Deref;
#[cfg(feature = "builtin-bytes")]
pub mod bytes;
pub mod container;
pub mod error;
pub mod extra;
pub mod input;
pub mod parser;
pub mod primitive;
#[cfg(feature = "error-recovery")]
pub mod recovery;
pub mod stream;
pub use aott_derive as derive;
#[cfg(feature = "serialization")]
pub mod ser;

#[cfg(feature = "builtin-text")]
pub mod text;

pub mod prelude {
        #[cfg(feature = "builtin-bytes")]
        pub use crate::bytes;
        pub use crate::derive::parser;
        pub use crate::error::{Error, PResult};
        pub use crate::extra;
        pub use crate::input::{
                ExactSizeInput, Input, InputOwned, InputType, SliceInput, StrInput,
        };
        pub use crate::parser::{Parser, ParserExtras};
        pub use crate::primitive::*;
        #[cfg(feature = "serialization")]
        pub use crate::ser::*;
        pub use crate::stream::Stream;
        #[cfg(feature = "builtin-text")]
        pub use crate::text;
}

/// This is a macro for declaring a type that is a parser function, using `impl Fn`;
/// It takes an $input, an $output, and an $extras type.
///
/// Example:
/// ```ignore
/// use aott::pfn_type;
///
/// fn myparser<I: InputType, E: ParserExtras<I>>(arg: i32) -> pfn_type!(I, (), E) {
///     move |input| {
///         // do some work...
///         println!("my argument: {arg}");
///         Ok(())
///     }
/// }
/// // equivalent to:
/// fn myparser<I: InputType, E: ParserExtras<I>>(arg: i32) ->
///     impl Fn(&mut Input<I, E>) -> PResult<I, (), E> {
///     /* -snip- */
/// }
/// ```
#[macro_export]
macro_rules! pfn_type {
        ($input:ty, $output:ty, $extras:ty) => {
                impl Fn(&mut $crate::input::Input<$input, $extras>) -> $crate::PResult<$input, $output, $extras>
        };
}

pub use error::PResult;

/// This enum allows for abstracting over references and owned values.
/// It is implemented kind of like [`Cow`], but without the lifetime.
/// If you want to have a [`Cow`]-esque API, you may want to use the type-alias [`MaybeRef`],
/// which is precisely created for storing either owned value `T`, or a reference `&'a T`.
///
/// [`Cow`]: `alloc::borrow::Cow`
pub enum MaybeDeref<T, R: Deref<Target = T>> {
        Ref(R),
        Val(T),
}

impl<T, R: Deref<Target = T>> From<T> for MaybeDeref<T, R> {
        fn from(value: T) -> Self {
                Self::Val(value)
        }
}
impl<'a, T> From<&'a T> for MaybeDeref<T, &'a T> {
        fn from(value: &'a T) -> Self {
                Self::Ref(value)
        }
}

pub type MaybeRef<'a, T> = MaybeDeref<T, &'a T>;

impl<'a, T> MaybeRef<'a, T> {
        pub fn borrow_as_t<'b: 'a>(&'b self) -> &'b T {
                match self {
                        Self::Ref(r) => r,
                        Self::Val(ref v) => v,
                }
        }
}

impl<T: Clone, R: Deref<Target = T>> MaybeDeref<T, R> {
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
