# AOTT - Aspect Of The TOKENS

AOTT is a Parser Combinator Framework, a library which allows you to munch
tokens declaratively, or imperatively, or both at the same time, how you would like.
AOTT gives you some neat primitives that you could build anything with,
and, if you want, builtin utilities for dealing with
text (feature flag `builtin-text`, on by default),
or bytes (feature flag `builtin-bytes`, off by default).

## Compatibility

AOTT is designed to be compatible with chumsky, so existing chumsky parsers will work / work with the minimum amount of changes (like removing lifetimes... *"ME HATE LIFETIME ARGUMENT! LIFETIME ARGUMENT BAD!"*).
A list of regexes to fix the parsers will be provided <del>never</del> soon.
About those lifetime arguments. I wanted to make a near lifetimeless chumsky
with functions as the primary unit of parsing (like in nom!), then I flomped into madness and now I have this. A near lifetimeless chumsky with functions. Yay.

## Thank you to

- [@zesterer](https://github.com/zesterer) for amazing work on [chumsky](https://github.com/zesterer/chumsky) and help in the Rust Community Discord server
- All [nom](https://github.com/rust-bakery/nom) contributors for their truly inspirational parser combinator framework library
- [@abs0luty](https://github.com/abs0luty) for his help at the early stages of the library and ideas for potential features.


