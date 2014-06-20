# Paws.rs

*An implementation of [Paws](http://ell.io/spec) in Rust*

## Building

    $ make

## Testing

    $ make test

## Running

    $ build/paws_rs

Current behavior: parses input and iterates `Execution::advance()` with a 'test'
symbol, printing each resulting combination.
