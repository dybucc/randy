//! The game module contains the core parts of the game, except for input and request handling.
//!
//! It contains the `init()` function to initialize and start the game loop, as well as the game
//! initialization message, some terminal configuration and the random number processor.

#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "It's best if the run() function is kept before any functions it itself uses."
)]

use std::{sync::LazyLock, thread::sleep, time::Duration};

use anyhow::Result;
use console::{pad_str, style, Term};
use fastrand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::frame::main_menu::{MainMenu, MainMenuAction};
use crate::frame::options::{OptionsMenu, OptionsMenuAction};
use crate::frame::prompt::nav_sliding_prompt;
use crate::frame::random_prompt::nav_input_prompt;
use crate::frame::{draw_menu, nav_menu};

/// This static variable holds the message to use for the system prompt on the request builder to
/// the chat completion request of the OpenRouter API. It is made static because the text is long
/// and it is thus best initialized the first time it is used.
static LLM_INPUT: LazyLock<&str> = LazyLock::new(|| {
    "You will answer only to \"Correct\" or \"Incorrect.\" These correspond to either a\
notification that a user got a number right in a number guessing game or not, respectively. Your\
task is to, depending on whether you were notified they got it right, or not, to return a\
cowboy-like answer to the user. Make it a short text. Include just your answer and nothing more.\
Don't include emoji or otherwise non-verbal content."
});

/// This structure holds information about the messages to send to the LLM in a chat completion
/// request to the OpenRouter API.
#[derive(Serialize, Deserialize)]
struct Messages {
    /// This field contains information about the content of the specific message in question.
    content: String,
    /// This field contains information about who is it that is supposed to be reporting the
    /// [`message`] field.
    role: Role,
}

impl Messages {
    /// This function creates a new message based on a given role for the chat exchange and the
    /// contents of the message in question.
    fn new(role: Role, content: &str) -> Self {
        Self {
            content: content.to_owned(),
            role,
        }
    }
}

/// This enum holds the variants to the final result of the user, to better transfer between
/// different parts of the stateful variable that the result of the current game is.
enum RandomResult {
    /// If the guess made by the user is correct, this variant will be used to report the status of
    /// the current game to other parts of the program.
    Correct,
    /// If the guess made by the user is inccorrect, this variant will be used to report the status
    /// of the current game to other parts of the program.
    Incorrect,
}

/// This structure is the main way of serializing information about the data we are interested in
/// for the chat completion request to the OpenRouter API.
#[derive(Serialize)]
struct Request {
    /// This field contains information about the sequence of messages to initially issue to the
    /// LLM.
    messages: Vec<Messages>,
    /// This field contains information about the model to be used in the request.
    model: String,
}

impl Request {
    /// This function creates a new chat completion request body solely with the information
    /// required by the program.
    fn new(guess: RandomResult, model: &str) -> Self {
        match guess {
            RandomResult::Correct => Self {
                model: model.to_owned(),
                messages: vec![
                    Messages::new(Role::System, *LLM_INPUT),
                    Messages::new(Role::User, "Correct"),
                ],
            },
            RandomResult::Incorrect => Self {
                model: model.to_owned(),
                messages: vec![
                    Messages::new(Role::System, *LLM_INPUT),
                    Messages::new(Role::User, "Incorrect"),
                ],
            },
        }
    }
}

/// This structure represents the response of a chat completion request to the OpenRouter API only
/// with the values that the program needs.
#[derive(Deserialize)]
struct Response {
    /// This field contains the vector of messages that the LLM has produced.
    choices: Vec<ResponseMessages>,
}

/// This structure holds information about the one-level indented message containing the responses
/// from the LLM.
#[derive(Deserialize)]
struct ResponseMessages {
    /// This field contains the actual responses from the LLM.
    message: Messages,
}

/// This enumeration represents the role in a chat exchange between a user and the LLM.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Role {
    /// This variant represents the role of the LLM.
    Assistant,
    /// This variant represents the role of the system prompt.
    System,
    /// This variant represents the role of the user.
    User,
}

/// Initializes the game state and handles literally everything. This is a `main()` function of
/// sorts though it is still called from main.rs.
///
/// This function specifically creates a new interface to the standard output, and a new rng
/// instance to avoid calling the thread local generator every time the loop runs for another
/// iteration.
///
/// # Errors
///
/// The function may return any one of the following errors:
///
/// - [`Regex::Error`]
/// - [`ureq::Error`]
/// - [`randyrand::ResponseError`]
pub fn run(model: Option<String>, api_key: &str) -> Result<()> {
    let term = Term::stdout();
    let mut model = model.unwrap_or_else(|| "featherless/qwerky-72b:free".to_owned());
    let mut main_menu = MainMenu::Play;
    let mut options_menu = OptionsMenu::Model;
    let ranged_re = Regex::new(r"\A\d+\.\.\d+\z")?;
    let random_re = Regex::new(r"\A\d+\z")?;
    let mut rng = Rng::new();

    loop {
        draw_menu(&term, &main_menu)?;

        match nav_menu(&term, &mut main_menu)? {
            MainMenuAction::Pass => continue,
            MainMenuAction::Finish => break,
            MainMenuAction::OptionsPage => options(&term, &mut options_menu, &mut model)?,
            MainMenuAction::StartGame => {
                let (guess, range_start, range_end) =
                    nav_input_prompt(&term, (&ranged_re, &random_re))?;

                let result = process_random((range_start, range_end), guess, &mut rng);
                let message = process_request(&term, &model, api_key, result)?;

                term.clear_screen()?;
                let (rows, cols) = term.size();
                for _ in 1..rows / 2 {
                    term.write_line("")?;
                }

                let output = pad_str(&message, cols as usize, console::Alignment::Center, None);
                term.write_line(&output)?;
                sleep(Duration::from_secs(5));

                break;
            }
        }
    }

    term.clear_screen()?;

    Ok(())
}

/// This function renders the options menu.
fn options(term: &Term, menu: &mut OptionsMenu, model: &mut String) -> Result<()> {
    loop {
        draw_menu(term, menu)?;

        match nav_menu(term, menu)? {
            OptionsMenuAction::ChangeModel => {
                nav_sliding_prompt(term, model)?;
            }
            OptionsMenuAction::GoBack => break,
            OptionsMenuAction::Pass => continue,
        }
    }

    Ok(())
}

/// This functions takes the role of number generator, as it takes both inputs from the user per
/// game, and both produces the number to be guessed within the given range, and matches the user
/// input to such number.
fn process_random(range: (usize, usize), input: usize, rng: &mut Rng) -> RandomResult {
    let random = rng.usize(range.0..=range.1);

    match input {
        _ if input == random => RandomResult::Correct,
        _ => RandomResult::Incorrect,
    }
}

/// This function builds a request body and processes a chat completion request to the OpenRouter
/// API.
fn process_request(
    term: &Term,
    model: &str,
    api_key: &str,
    result: RandomResult,
) -> Result<String> {
    let request_body = Request::new(result, model);
    let agent = Agent::new_with_defaults();
    let (rows, cols) = term.size();
    let (dot1, dot2, dot3) = (
        format!("{}", style(".").bold()),
        format!("{}", style("..").bold()),
        format!("{}", style("...").bold()),
    );

    term.clear_screen()?;
    term.hide_cursor()?;

    for _ in 1..rows / 2 - 1 {
        term.write_line("")?;
    }

    let output = format!("{}", style("Processing").bold());
    let output = pad_str(&output, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;
    sleep(Duration::from_millis(100));

    loop {
        let output = pad_str(&dot1, cols as usize, console::Alignment::Center, None);
        term.write_line(&output)?;

        let response = agent
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .send_json(&request_body);

        term.move_cursor_up(1)?;
        term.clear_line()?;
        let output = pad_str(&dot2, cols as usize, console::Alignment::Center, None);
        term.write_line(&output)?;

        match response {
            Ok(response) => {
                let response: Response = response.into_body().read_json()?;
                let output = &response
                    .choices
                    .last()
                    .expect("empty vector")
                    .message
                    .content;

                if !output.is_empty() {
                    break Ok(output.to_owned());
                }
            }
            Err(err) => {
                break Err(err.into());
            }
        }

        sleep(Duration::from_millis(100));
        term.move_cursor_up(1)?;
        term.clear_line()?;
        let output = pad_str(&dot3, cols as usize, console::Alignment::Center, None);
        term.write_line(&output)?;
        sleep(Duration::from_millis(100));
        term.move_cursor_up(1)?;
        term.clear_line()?;
    }
}
