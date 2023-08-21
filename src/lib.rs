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

#[inline(never)]
fn do_i_hate_llvm(h: &i32) -> bool {
        drop(h);
        true
}
