//! The integration-test suite, compiled under edition 2021. See Cargo.toml.
#![cfg(test)]
#![allow(clippy::all)]

#[path = "../../alg.rs"]
mod alg;
#[path = "../../attribute.rs"]
mod attribute;
#[path = "../../compound.rs"]
mod compound;
#[path = "../../dispatch.rs"]
mod dispatch;
#[path = "../../expressions.rs"]
mod expressions;
#[path = "../../features.rs"]
mod features;
#[path = "../../macros.rs"]
mod macros;
#[path = "../../operators.rs"]
mod operators;
#[path = "../../passthrough.rs"]
mod passthrough;
#[path = "../../fuzz_corpus_f32.rs"]
mod fuzz_corpus_f32;
