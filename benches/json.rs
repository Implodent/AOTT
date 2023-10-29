#![feature(test)]

use std::collections::HashMap;

extern crate aott;
extern crate test;

use aott::prelude::*;
use test::black_box;
use test::Bencher;

#[derive(Debug, Clone)]
pub enum Value<'a> {
        String(&'a str),
        Number(f64),
        Boolean(bool),
        Array(Vec<Value<'a>>),
        Object(HashMap<&'a str, Value<'a>>),
        Null,
}

mod naive {
        use super::*;

        fn json<'a>() -> impl Parser<&'a str, Value<'a>, extra::Err<&'a str>> {
                recursive(|value| {
                        let digits = || one_of('0'..='9').repeated();

                        let int = one_of('1'..='9')
                                .then(one_of('0'..='9').repeated())
                                .ignored()
                                .or(just('0').ignored())
                                .ignored();

                        let frac = just('.').then(digits());

                        let exp = one_of("eE").then(one_of("+-").optional()).then(digits());

                        let number = just('-')
                                .optional()
                                .then(int)
                                .then(frac.optional())
                                .then(exp.optional())
                                .slice()
                                .map(|x: &str| x.parse().unwrap());

                        let escape = || just('\\').then_ignore(one_of("\\/\"bfnrt"));

                        let string = || {
                                none_of("\\\"")
                                        .or(escape())
                                        .repeated()
                                        .slice()
                                        .delimited_by(just('"'), just('"'))
                        };

                        let array = value
                                .clone()
                                .separated_by(just(','))
                                .collect()
                                .padded()
                                .delimited_by(just('['), just(']'));

                        let member = string().then_ignore(just(':').padded()).then(value);
                        let object = member
                                .separated_by(just(',').padded())
                                .collect()
                                .padded()
                                .delimited_by(just('{'), just('}'));

                        choice((
                                just("null").to(Value::Null),
                                just("true").to(Value::Boolean(true)),
                                just("false").to(Value::Boolean(false)),
                                number.map(Value::Number),
                                string().map(Value::String),
                                array.map(Value::Array),
                                object.map(Value::Object),
                        ))
                        .padded()
                })
        }

        pub fn parse(input: &str) -> Result<Value, extra::Simple<&str>> {
                json().parse(input)
        }
}

const INPUT: &str = include_str!("sample.json");

#[bench]
fn naive(b: &mut Bencher) {
        let input = black_box(INPUT.trim());
        println!("{input}");
        println!("{:?}", black_box(naive::parse(input)).unwrap());
        b.iter(|| black_box(naive::parse(black_box(input))).unwrap())
}
