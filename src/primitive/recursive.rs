use std::cell::OnceCell;

use crate::{
        input::InputType,
        parser::{Parser, ParserExtras},
};

type OnceParser<'a, I, O, E> = OnceCell<Box<dyn Parser<I, O, E> + 'a>>;

pub struct Recursive<'a, I: InputType, O, E: ParserExtras<I>>(
        RecursiveImp<OnceParser<'a, I, O, E>>,
);

enum RecursiveImp<T> {
        Owned(crate::sync::RefC<T>),
        Unowned(crate::sync::RefW<T>),
}

impl<'a, I: InputType, O, E: ParserExtras<I>> Recursive<'a, I, O, E> {
        fn cell(&self) -> crate::sync::RefC<OnceParser<'a, I, O, E>> {
                match &self.0 {
                        RecursiveImp::Owned(own) => own.clone(),
                        RecursiveImp::Unowned(unown) => unown
                                .upgrade()
                                .expect("recursive parser used before definition"),
                }
        }
        pub fn declare() -> Self {
                Self(RecursiveImp::Owned(crate::sync::RefC::new(OnceCell::new())))
        }

        pub fn define(&mut self, parser: impl Parser<I, O, E> + 'a) {
                self.cell()
                        .set(Box::new(parser))
                        .unwrap_or_else(|_| panic!("Parser defined more than once"))
        }
}

impl<'a, I: InputType, O, E: ParserExtras<I>> Clone for Recursive<'a, I, O, E> {
        fn clone(&self) -> Self {
                Self(match &self.0 {
                        RecursiveImp::Owned(own) => {
                                RecursiveImp::Unowned(crate::sync::RefC::downgrade(own))
                        }
                        RecursiveImp::Unowned(unown) => {
                                RecursiveImp::Unowned(crate::sync::RefW::clone(unown))
                        }
                })
        }
}

impl<'a, I: InputType, O, E: ParserExtras<I>> Parser<I, O, E> for Recursive<'a, I, O, E> {
        fn check_with(&self, input: &mut crate::input::Input<I, E>) -> crate::PResult<I, (), E> {
                self.cell()
                        .get()
                        .expect("Recursive parser used before definition")
                        .as_ref()
                        .check_with(input)
        }

        fn parse_with(&self, input: &mut crate::input::Input<I, E>) -> crate::PResult<I, O, E> {
                self.cell()
                        .get()
                        .expect("Recursive parser used before definition")
                        .as_ref()
                        .parse_with(input)
        }
}

pub fn recursive<'a, I: InputType + 'a, O: 'a, E: ParserExtras<I> + 'a, P: Parser<I, O, E> + 'a>(
        def: impl Fn(Recursive<'a, I, O, E>) -> P + 'a,
) -> impl Parser<I, O, E> + 'a {
        let mut rec = Recursive::declare();
        rec.define(def(rec.clone()));
        rec
}
