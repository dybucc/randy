//! The game module contains the core parts of the game, except for input and request handling.
//!
//! It contains the `init()` function to initialize and start the game loop, as well as the game
//! initialization message, some terminal configuration and the random number processor.

use anyhow::Result;
use clap::Parser;
use console::{style, Term};
use fastrand::Rng;
use regex::Regex;
use serde::Deserialize;

use crate::input::{exit, take_input, take_ranged_input};
use crate::messages::{process_message, response_error};

/// This struct holds information about the application when it comes to the command-line argument
/// parser of choice, which is clap. It uses the derive attribute and multiple other attributes to
/// set up the different commands, as that was found to be the simplest way of accomplishing what
/// was set out to do.
#[derive(Parser)]
#[command(name = "randy", version, about)]
#[command(next_line_help = true)]
struct Cli {
    /// The OpenRouter API key to provide for the AI-based responses.
    ///
    /// This argument is only required if the environment variable OPENROUTER_API_KEY is not set
    /// with the corresponding API key. Otherwise, you will have to specify this option.
    #[arg(long)]
    #[arg(env = "OPENROUTER_API_KEY", value_name = "YOUR_API_KEY")]
    api_key: String,
    /// The model name to produce the response; DeepSeek's V3 by default.
    ///
    /// Models are processed by the string right below their public brand name in their respective
    /// OpenRouter model page. If you want to set it to anything other than the default free model,
    /// you will have to use that name.
    #[arg(short, long, requires = "api_key", value_parser = verify_model)]
    #[arg(env = "OPENROUTER_MODEL", value_name = "MODEL_NAME")]
    model: Option<String>,
}

/// It makes up one of the fields the request to fetch models from the OpenRouter API requires. This
/// structure doesn't support all of the mandatory and optional fields because the request is only
/// interested in the model id.
#[derive(Deserialize)]
struct Data {
    /// This field contains the name to be used on post requests in the model field for OpenRouter
    /// POST API requests.
    id: String,
}

/// This structure contains the main form of the response returned by an OpenRouter API request for
/// the list of all models available for use in the API.
#[derive(Deserialize)]
struct ModelResponse {
    /// This field contains the only part of the response that the OpenRouter API returns on their
    /// list all models GET request. It is a list of objects that is abstracted as another struct to
    /// deserialize.
    data: Vec<Data>,
}

/// This enum holds the variants to the final result of the user, to better transfer between
/// different parts of the stateful variable that the result of the current game is.
pub(crate) enum RandomResult {
    /// If the guess made by the user is correct, this variant will be used to report the status of
    /// the current game to other parts of the program.
    Correct,
    /// If the guess made by the user is inccorrect, this variant will be used to report the status
    /// of the current game to other parts of the program.
    Incorrect,
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
/// - Regex::Error
/// - io::Error
/// - dialoguer::Error
/// - randyrand::ResponseError
#[expect(
    clippy::missing_panics_doc,
    reason = "The panic's only due to the unwrapping of a regular expression. It's been tested, and it's been proven to be syntactically correct."
)]
pub fn init() -> Result<()> {
    let term = Term::stdout();
    let mut rng = Rng::new();
    let cli = Cli::parse();
    let ranged_re = Regex::new(r"\A\d+\.\.\d+\z").unwrap();
    let model = cli
        .model
        .unwrap_or("deepseek/deepseek-chat-v3-0324:free".to_string());

    // show the init message
    init_message(&term)?;

    // game loop
    loop {
        // prompt for a range of inputs
        let range = take_ranged_input(&term, &ranged_re)?;

        // prompt for an input
        let input = take_input(&term, &range)?;

        // run the rng within the given range and check the user's input
        let result = process_random(range, input, &mut rng);

        // process the message query to say that the user won or not
        match process_message(result, &cli.api_key, &model) {
            Ok(output) => {
                term.write_line(&format!("{}", style(output).bold()))?;

                if !exit(&term)? {
                    term.clear_screen()?;
                    break Ok(());
                }

                term.clear_screen()?;
            }
            Err(err) => break Err(response_error(err)),
        }
    }
}

/// This function initializes the message to be used at the start of the program, as well as a few
/// other fallible operations. Among these, the screen is cleared and the cursor is hidden. The
/// title of the console window is also set to the name of the game.
fn init_message(term: &Term) -> Result<()> {
    const MSG: &str = "Welcome to the game of randy";
    let msg = style(MSG).bold();

    term.clear_screen()?;
    term.set_title("randy");
    term.hide_cursor()?;

    term.write_line(&format!("{}", msg))?;
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

/// This function serves as a value parser for the command line argument parser in the `model`
/// field. It basically makes a request to the OpenRouter API to retrieve the list of available
/// models to use through their API and checks if the string passed by clap matches any one of the
/// strings retrieved in the request.
fn verify_model(string: &str) -> Result<String, String> {
    let request = ureq::get("https://openrouter.ai/api/v1/models").call();

    match request {
        Ok(response) => {
            let response: ModelResponse = response.into_body().read_json().unwrap();
            let mut output =
                String::from("The requested model could not be found with the OpenRouter API.");

            for Data { id } in response.data {
                if id == string {
                    output = string.to_string();
                    return Ok(output);
                }
            }

            Err(output)
        }
        Err(_) => Err(
            "There's been an error checking the requested model with the OpenRouter API."
                .to_string(),
        ),
    }
}
