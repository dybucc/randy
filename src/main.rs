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
use clap::Parser;
use serde::Deserialize;

/// This struct holds information about the application when it comes to the command-line argument
/// parser of choice, which is clap.
///
/// It uses the derive attribute and multiple other attributes to set up the different commands, as
/// that was found to be the simplest way of accomplishing what was set out to do.
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
    /// The model name to produce the response; Qwerky 72B by default.
    ///
    /// Models are processed by the string right below their public brand name in their respective
    /// OpenRouter model page. If you want to set it to anything other than the default free model,
    /// you will have to either use that name in the command-line, the environment variable or
    /// change it in the menu once in-game.
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    randyrand::run(cli.model, &cli.api_key)
}

/// This function serves as a value parser for the command line argument parser in the `model`
/// field. It basically makes a request to the OpenRouter API to retrieve the list of available
/// models to use through their API and checks if the string passed by clap matches any one of the
/// strings retrieved in the request.
fn verify_model(string: &str) -> Result<String, String> {
    let request = ureq::get("https://openrouter.ai/api/v1/models").call();

    match request {
        Ok(response) => {
            let response: ModelResponse =
                response.into_body().read_json().expect("response failed");
            let mut output =
                String::from("The requested model could not be found with the OpenRouter API.");

            for Data { id } in response.data {
                if id == string {
                    string.clone_into(&mut output);
                    return Ok(output);
                }
            }

            Err(output)
        }
        Err(_) => Err(
            "There's been an error checking the requested model with the OpenRouter API."
                .to_owned(),
        ),
    }
}
