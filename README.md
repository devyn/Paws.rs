# Paws.rs

*An implementation of [Paws](http://ell.io/spec) in Rust*

## Building

    $ ./build.sh

At the moment, there are no configuration environment variables. I might switch
to a Makefile in the future. Also, this always runs tests.

## Running

    $ build/paws_rs

Type in your input, hit EOF, and look at the output straight from the parser.
That's all for now.

## Examples

    $ build/paws_rs
    happy (happy happy) "happy happy" {happy “happy happy
    happy”}
    ~[Symbol(~"happy"), Expression(~[Symbol(~"happy"), Symbol(~"happy")]), Symbol(~"happy happy"), Execution(~[Symbol(~"happy"), Symbol(~"happy happy\nhappy")])]

    $ build/paws_rs
    this is
    going to be an error{
    Parse error: <stdin>:3:1: expected '}' before end-of-input

    $ build/paws_rs
    blah blah} error
    Parse error: <stdin>:1:10: unexpected terminator '}'
