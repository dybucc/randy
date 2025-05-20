use std::{collections::HashMap, sync::LazyLock};

use anyhow::Result;
use clap::{Arg, Parser};
use console::{style, Term};
use fastrand::Rng;
use serde::Deserialize;

use crate::input::{take_input, take_ranged_input};
use crate::messages::{self, process_message};

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The JSON schema requires this order."
)]
#[derive(Deserialize)]
struct Architecture {
    input_modalities: Vec<String>,
    output_modalities: Vec<String>,
    tokenizer: String,
    instruct_type: String,
}

// TODO finish configuring clap with the desired options
#[derive(Parser)]
#[command(name = "randy", version, about)]
#[command(next_line_help = true)]
struct Cli {
    /// The OpenRouter API key to provide for the AI-based responses.
    ///
    /// This argument is only required if the environment variable OPENROUTER_API is not set with
    /// the corresponding API key. Otherwise, you will have to specify this option.
    #[arg(long, env = "OPENROUTER_API")]
    api_key: String,
    /// The model name to produce the response; DeepSeek's V3 by default.
    ///
    /// Models are processed by the string right below their public brand name in their respective
    /// OpenRouter model page. If you want to set it to anything other than the default free model,
    /// you will have to use that name.
    #[arg(short, long, value_parser = verify_model, requires = "api_key")]
    model: Option<String>,
}

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The JSON schema requires this order."
)]
#[derive(Deserialize)]
struct Data {
    id: String,
    name: String,
    created: f64,
    description: String,
    architecture: Architecture,
    top_provider: TopProvider,
    pricing: Pricing,
    context_length: f64,
    hugging_face_id: String,
    per_request_limits: HashMap<String, String>,
    supported_parameters: Vec<String>,
}

#[derive(Deserialize)]
struct ModelResponse {
    data: Vec<Data>,
}

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The JSON schema requires this order."
)]
#[derive(Deserialize)]
struct Pricing {
    prompt: String,
    completion: String,
    image: String,
    request: String,
    input_cache_read: String,
    input_cache_write: String,
    web_search: String,
    internal_reasoning: String,
}

pub(crate) enum RandomResult {
    Correct,
    Incorrect,
}

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The JSON schema requires this order."
)]
#[derive(Deserialize)]
struct TopProvider {
    is_moderated: bool,
    context_length: f64,
    max_completion_tokens: f64,
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
    init_message(&term);

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
            term.write_line(&format!("{}", style(output).bold()));
            Ok(())
        }
        Err(e) => Err(messages::response_error(e)),
    }
}

fn init_message(term: &Term) {
    const MSG: &str = "Welcome to the game of randy";
    let msg = style(MSG).bold();

    term.clear_screen();
    term.set_title("randy");
    term.hide_cursor();

    term.write_line(&format!("{}", msg));
}

fn process_random(range: (usize, usize), input: usize, mut rng: Rng) -> RandomResult {
    let random = rng.usize(range.0..range.1);

    match input {
        _ if input == random => RandomResult::Correct,
        _ => RandomResult::Incorrect,
    }
}

fn verify_model(s: &str) -> Result<Option<String>, String> {
    let cli = Cli::parse();

    let request = ureq::get("https://openrouter.ai/api/v1/models")
        .header("Authentication", format!("Bearer {}", cli.api_key))
        .call();

    match request {
        Ok(r) => {
            let response: ModelResponse = r.into_body().read_json().unwrap();
            let mut output = String::from("The model couldn't be found in OpenRouter's API.");

            for model in response.data {
                if model.name == s {
                    output = s.to_string();
                    break;
                }
            }

            Ok(Some(output))
        }
        Err(e) => Err("There's been an error processing the list of models.".to_string()),
    }
}
