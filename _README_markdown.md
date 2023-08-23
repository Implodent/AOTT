
# Table of Contents

1.  [AOTT - Aspect Of The TOKENS](#org13b2bfb)
    1.  [Compatibility](#orgab23c87)
    2.  [Thank you to](#org110d52a)
    3.  [TO-DO list / Roadmap <code>[0/2]</code>](#org668c302)


<a id="org13b2bfb"></a>

# AOTT - Aspect Of The TOKENS

AOTT is a Parser Combinator Framework, a library which allows you to munch
tokens declaratively, or imperatively, or both at the same time, how you would like.
AOTT gives you some neat primitives that you could build anything with,
and, if you want, builtin utilities (NOT YET IMPLEMENTED!) for dealing with
text (feature flag `builtin-text`, on by default),
or bytes (feature flag `builtin-bytes`, off by default).


<a id="orgab23c87"></a>

## Compatibility

AOTT is designed to be compatible with chumsky, so existing chumsky parsers will work / work with the minimum amount of changes (like removing lifetimes&#x2026; *&ldquo;ME HATE LIFETIME ARGUMENT! LIFETIME ARGUMENT BAD!&rdquo;*).
A list of regexes to fix the parsers will be provided <del>never</del> soon.
About those lifetime arguments. I wanted to make a near lifetimeless chumsky
with functions as the primary unit of parsing (like in nom!), then I flomped into madness and now I have this. A near lifetimeless chumsky with functions. Yay.


<a id="org110d52a"></a>

## Thank you to

-   [@zesterer](https://github.com/zesterer) for amazing work on [chumsky](https://github.com/zesterer/chumsky) and help in the Rust Community Discord server
-   All [nom](https://github.com/rust-bakery/nom) contributors for their truly inspirational parser combinator framework library
-   [@abs0luty](https://github.com/abs0luty) for his help at the early stages of the library and ideas for potential features.


<a id="org668c302"></a>

## TO-DO list / Roadmap <code>[0/2]</code>

-   [-] Full feature parity with chumsky <code>[1/2]</code>
    -   [X] Primitives <code>[4/4]</code>
        -   [X] just (and container types)
        -   [X] filter, filter<sub>map</sub>, select
        -   [X] group (renamed to tuple)
        -   [X] choice
    -   [-] All Parser:: functions <code>[2/5]</code>
        -   [X] .or
        -   [X] .map, .to
        -   [ ] .try<sub>map</sub>
        -   [ ] .ignored (make parser return `()`)
        -   [ ] .then (dont implement, useless function), .then<sub>ignore</sub>, .ignore<sub>then</sub>
-   [ ] Error recovery?

