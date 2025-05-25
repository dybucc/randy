//! The library components of the game. They allow initializing the game, taking input, processing a
//! random number and fetching a response from the OpenRouter API.
//!
//! The starting point of the library is the game.rs file, which contains the main game loop.

#![expect(
    unused_crate_dependencies,
    reason = "clap is not used in the library crate, but it is used in the binary crate."
)]

mod frame;
mod game;

pub use game::run;
