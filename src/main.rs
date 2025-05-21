//! # randy
//!
//! This crate is a game about guessing a number between a user-specified range and picking from
//! that range a number to guess. It only runs on Linux because that's how I will it to be.
//! It is inspired on the Rust book's initial `guessing-game` project.
//!
//! You, in turn, get an answer that is cowboy-like in manner, and hopefully doesn't deviate from an
//! otherwise non-AI generated answer.
//!
//! The answer is retrieved through the OpenRouter API by means of request calls and simple
//! deserialization and serialization code. The library is really small and only covers the use
//! cases I found for this particular project, so there's no full coverage of the platform's API.

#![cfg(target_os = "linux")]
#![expect(
    unused_crate_dependencies,
    reason = "The dependencies are used in the library crate."
)]

use anyhow::Result;
use randyrand::init;

fn main() -> Result<()> {
    init()
}
