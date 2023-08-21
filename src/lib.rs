#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use core::ops::Deref;

pub mod error;
pub mod input;
pub mod parser;
pub mod primitive;
pub mod text;

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
