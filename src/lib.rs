//! The library components of the game. They allow initializing the game, taking input, processing a
//! random number and fetching a response from the OpenRouter API.
//!
//! The starting point of the library is the game.rs file, which contains the main game loop.

#![expect(unused, reason = "Temporary allow during development.")]
#![expect(
    clippy::cargo_common_metadata,
    reason = "The package has not yet been pushed to a remote."
)]

mod game;
mod input;
mod messages;

pub use game::init;
