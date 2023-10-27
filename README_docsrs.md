# AOTT - Aspect Of The Tokens

[![crates.io](https://img.shields.io/crates/v/aott.svg)](https://crates.io/crates/aott)
[![crates.io](https://docs.rs/aott/badge.svg)](https://docs.rs/aott)
[![License](https://img.shields.io/crates/l/aott.svg)](https://github.com/Implodent/AOTT)
[![Discord](https://img.shields.io/discord/1027291338548445304)](https://discord.gg/k9vTZNtPGX)
    
AOTT is a parser combinator framework - a library that contains utilities for painlessly creating parsers.

It also includes a few built-in utilities,
like parsers for working with text (see [`text`](./src/text.rs) module; feature gated under flag `builtin-text`, enabled by default),
and several handy decoders for working with bytes (see [`bytes`](./src/bytes.rs) module; feature gated under flag `builtin-bytes`, not enabled by default).
        
## Features
- 🪄 **Expressive built-in combinators** that make writing your parser a joy
- 🎛 **Fully generic** across input, token, output, span, and error types
- 📑 **Zero-copy parsing** built into the library from the start; minimizing your parser's need to allocate
- 📖 **Text- and bytes-oriented parsers** for text and bytes inputs (i.e: `&[u8]` and `&str`)

## Example parser
See [`examples/brainfuck.rs`](./examples/brainfuck.rs) for a full interpreter (`cargo run --example brainfuck - examples/sample.bf`).

```rust
use aott::prelude::*;

#[derive(Clone, Debug)]
enum Instruction {
        Left,
        Right,
        Increment,
        Decrement,
        Read,
        Write,
        Loop(Vec<Self>),
}

#[parser]
fn parse(input: &str) -> Vec<Instruction> {
        choice((
                // Basic instructions are just single characters!
                just('<').to(Instruction::Left),
                just('>').to(Instruction::Right),
                just('+').to(Instruction::Increment),
                just('-').to(Instruction::Decrement),
                just(',').to(Instruction::Read),
                just('.').to(Instruction::Write),
                // recursion is easy: just put in the function as is!
                delimited(just('['), parse, just(']')).map(Instruction::Loop),
        ))
        // Brainfuck is sequential, so we parse as many instructions as is possible
        .repeated()
        .collect::<Vec<_>>()
        .parse_with(input)
}
```

## *What* are parser combinators?
Parser combinators are a technique for implementing parser by defining them in terms of other parsers.

That means you construct smaller parsers, for example, for a string, and then make bigger and bigger parsers, like expressions, then statements, then functions and so on and so on. This way of creating parsers (from small to big) is called the [Recursive Descent](https://en.wikipedia.org/wiki/Recursive_descent_parser) strategy.

Parser combinators are kind of similar to Rust's [`Iterator` trait](https://doc.rust-lang.org/std/iter/trait.Iterator.html) to define the parsing algorithm: the type-driven API of `Iterator` makes it much more difficult to make mistakes, and much easier to encode complicated iteration logic than if one were to write the same code by hand.
The same is true for parser combinators.

## *Why* should I even use parser combinators?
Writing parsers with good error recovery is made exponentially more difficult the more features you add into the parser.
It requires understanding intricacies of the recursive descent algorithm, **and then** implementing recovery strategies on top of that.

If you are developing, say, a programming language, or even just starting out, you will find yourself making a lot of small changes very often. And if you're writing a parser by hand, there will be a slow and painful step of refactoring your parser for these changes.

Parser combinators solve both problems by providing an ergonomic API that allows for rapidly iterating upon a syntax.

Parser combinators are also a great fit for domain-specific languages (DSLs), for which there is no existing parser.
Writing a reliable, fault-tolerant parser for such situations can go from being a weeks-long task to a half-hour long task, if you employ a decent parser combinator library.
