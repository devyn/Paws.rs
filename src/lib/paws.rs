//! The Rust implementation of [Paws](http://paws.mu) and its Nucleus.
//!
//! **Paws' Nucleus** is a programming language designed to be an *abstractive*
//! target.  Instead of writing in it directly (like Ruby), or compiling or
//! translating to it (LLVM IR, sometimes JavaScript, etc.), you create new
//! languages on top of it by creating your own abstractions within it.
//!
//! It provides an execution model, a data model, and a concurrency model, and
//! is specified in such a way that implicit parallelism is possible. There is
//! also a syntax to be used with the Nucleus that is implemented here, called
//! **cPaws**.
//!
//! See the [specification](http://ell.io/spec) for more information.

#![crate_name = "paws"]
#![crate_type = "lib"]

#![feature(globs)]
#![feature(phase)]

#![warn(missing_doc)]

extern crate native;
extern crate term;

#[phase(plugin, link)]
extern crate log;

/*DEBUG*/ extern crate time;

pub mod cpaws;
pub mod object;
pub mod nuketype;
pub mod script;
pub mod machine;
pub mod system;
pub mod specification;

mod util;
