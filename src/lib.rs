//! The library components of the game. They allow initializing the game, taking input, processing a
//! random number and fetching a response from the OpenRouter API.
//!
//! The starting point of the library is the game.rs file, which contains the main game loop.

// #![expect(unused, reason = "Temporary allow during development.")]

mod frame;
mod game;
mod input;
mod messages;

pub use game::run;
