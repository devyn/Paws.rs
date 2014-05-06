# Paws.rs

*An implementation of [Paws](http://ell.io/spec) in Rust*

## Building

    rustc src/lib/paws.rs
    rustc -L . src/bin/paws_rs.rs

I'm still looking into the best way to automate this, since the library filename
Rust generates is based on a hash and isn't fixed.

## Running

    ./paws_rs

Type in your input, hit EOF, and look at the output straight from the parser.
That's all for now.

## Examples

    $ ./paws_rs
    happy (happy happy) "happy happy" {happy “happy happy
    happy”}
    ~[Symbol(~"happy"), Expression(~[Symbol(~"happy"), Symbol(~"happy")]), Symbo
    l(~"happy happy"), Execution(~[Symbol(~"happy"), Symbol(~"happy happy\n    h
    appy")])]

    $ ./paws_rs
    this is
    going to be an error{
    Parse error: <stdin>:3:1: expected '}' before end-of-input

    $ ./paws_rs
    blah blah} error
    Parse error: <stdin>:1:10: unexpected terminator '}'
