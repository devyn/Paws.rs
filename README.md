# Paws.rs

*An implementation of [Paws](http://ell.io/spec) in Rust*

## Building

    $ make

## Testing

    $ make test

## Running

    $ build/paws_rs --help

To enable logging:

    $ RUST_LOG=4 build/paws_rs

There are examples to run in `examples/`. For example,

    $ build/paws_rs examples/01.hello.world.paws

Paws.rs will run forever by default. To have it end itself when there's no more
work to be done, use the `--no-stall` option:

    $ build/paws_rs --no-stall examples/01.hello.world.paws
