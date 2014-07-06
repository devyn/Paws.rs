# Paws.rs

*An implementation of [Paws](http://ell.io/spec) in Rust*

## Building

    $ make

## Testing

    $ make test

## Running

Without logging:

    $ build/paws_rs < path/to/file.paws

With logging:

    $ RUST_LOG="paws=4" build/paws_rs < path/to/file.paws

There are examples to run in `examples/`. You'll have to terminate Paws.rs
yourself (the usual way, ^C), or add an `implementation stop[]` call in there
somewhere; otherwise it will run forever.
