//! The integration-test suite, compiled under edition 2021. See Cargo.toml;
//! `reassoc/tests/suite_layout.rs` keeps this list complete.
#![cfg(test)]
#![allow(clippy::all)]
// `rustfmt::skip` on each: rustfmt would otherwise format the shared files as
// edition-2021 modules (`t.1 .1`, different block breaking) and fight the
// owning package's 2024 formatting.

#[rustfmt::skip]
#[path = "../../../reassoc/tests/alg.rs"]
mod alg;
#[rustfmt::skip]
#[path = "../../../reassoc/tests/attribute.rs"]
mod attribute;
#[rustfmt::skip]
#[path = "../../../reassoc/tests/compound.rs"]
mod compound;
#[rustfmt::skip]
#[path = "../../../reassoc/tests/dispatch.rs"]
mod dispatch;
#[rustfmt::skip]
#[path = "../../../reassoc/tests/expressions.rs"]
mod expressions;
#[rustfmt::skip]
#[path = "../../../reassoc/tests/features.rs"]
mod features;
#[rustfmt::skip]
#[path = "../../../reassoc/tests/fuzz_corpus_f32.rs"]
mod fuzz_corpus_f32;
#[rustfmt::skip]
#[path = "../../../reassoc/tests/macros.rs"]
mod macros;
#[rustfmt::skip]
#[path = "../../../reassoc/tests/operators.rs"]
mod operators;
#[rustfmt::skip]
#[path = "../../../reassoc/tests/passthrough.rs"]
mod passthrough;
