use anyhow::Result;
use clap::Parser;
use console::{style, Term};
use fastrand::Rng;
use serde::Deserialize;

use crate::input::{take_input, take_ranged_input};
use crate::messages::{process_message, response_error};

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

#[derive(Deserialize)]
struct Data {
    id: String,
}

#[derive(Deserialize)]
struct ModelResponse {
    data: Vec<Data>,
}

pub(crate) enum RandomResult {
    Correct,
    Incorrect,
}

/// Initializes the game state and handles literally everything. This is a `main()` function of
/// sorts though it is still called from main.rs.
///
/// This function specifically creates a new interface to the standard output, and a new rng
/// instance to avoid calling the thread local generator every time the loop runs for another
/// iteration.
pub fn init() -> Result<()> {
    let term = Term::stdout();
    let rng = Rng::new();
    let cli = Cli::parse();

    // show the init message
    init_message(&term)?;

    // prompt for a range of inputs TODO validate inputs
    let range = take_ranged_input(&term);

    // prompt for an input TODO validate inputs
    let input = take_input(&term);

    // run the rng within the given range and check the user's input
    let result = process_random(range, input, rng);

    // process the message query to say that the user won or not
    match process_message(
        result,
        &cli.api_key,
        &cli.model
            .unwrap_or("deepseek/deepseek-chat-v3-0324:free".to_string()),
    ) {
        Ok(output) => {
            term.write_line(&format!("{}", style(output).bold()))?;
            Ok(())
        }
        Err(e) => Err(response_error(e)),
    }
}

fn init_message(term: &Term) -> Result<()> {
    const MSG: &str = "Welcome to the game of randy";
    let msg = style(MSG).bold();

    term.clear_screen()?;
    term.set_title("randy");
    term.hide_cursor()?;

    term.write_line(&format!("{}", msg))?;
    Ok(())
}

fn process_random(range: (usize, usize), input: usize, mut rng: Rng) -> RandomResult {
    let random = rng.usize(range.0..range.1);

    match input {
        _ if input == random => RandomResult::Correct,
        _ => RandomResult::Incorrect,
    }
}

fn verify_model(s: &str) -> Result<String, String> {
    let request = ureq::get("https://openrouter.ai/api/v1/models").call();

    match request {
        Ok(r) => {
            let response: ModelResponse = r.into_body().read_json().unwrap();
            let mut output =
                String::from("The requested model could not be found with the OpenRouter API.");

            for Data { id } in response.data {
                if id == s {
                    output = s.to_string();
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
