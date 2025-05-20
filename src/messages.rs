use std::{sync::LazyLock, time::Duration};

use anyhow::{Error, Result};
use console::style;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::game::RandomResult;

static LLM_INPUT: LazyLock<&str> = LazyLock::new(|| {
    "You will answer only to \"Correct\" or \"Incorrect.\" These correspond to either a\
notification that a user got a number right in a number guessing game or not, respectively. Your\
task is to, depending on whether you were notified they got it right, or not, to return a\
cowboy-like answer to the user. Make it a short text. Include just your answer and nothing more.\
Don't include emoji or otherwise non-verbal content."
});

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The JSON schema needs the fields to be in this order."
)]
#[derive(Serialize, Deserialize)]
struct Message {
    role: Role,
    content: String,
}

impl Message {
    fn new(role: Role, content: &str) -> Self {
        Self {
            role,
            content: content.to_string(),
        }
    }
}

#[derive(Serialize)]
struct Request {
    messages: Vec<Message>,
    model: String,
}

impl Request {
    fn new(input: RandomResult, model: &str) -> Self {
        match input {
            RandomResult::Correct => Self {
                messages: vec![
                    Message::new(Role::System, *LLM_INPUT),
                    Message::new(Role::User, "Correct"),
                ],
                model: model.to_string(),
            },
            RandomResult::Incorrect => Self {
                messages: vec![
                    Message::new(Role::System, *LLM_INPUT),
                    Message::new(Role::User, "Incorrect"),
                ],
                model: model.to_string(),
            },
        }
    }
}

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The JSON schema needs the fields to be in this order."
)]
#[derive(Serialize, Deserialize)]
struct Response {
    id: String,
    choices: Vec<ResponseChoices>,
    created: usize,
    model: String,
    object: ResponseObject,
}

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The JSON schema needs the fields to be in this order."
)]
#[derive(Serialize, Deserialize)]
struct ResponseChoices {
    finish_reason: String,
    native_finish_reason: String,
    message: Message,
}

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "It's easier to maintain if the errors are in the same order as the ones specified in the OpenRouter docs."
)]
#[derive(thiserror::Error, Debug)]
enum ResponseError {
    #[error("{}", style("bad request").bold().underlined())]
    BadRequest,
    #[error("{}", style("invalid credentials").bold().underlined())]
    InvalidCredentials,
    #[error("{}", style("insufficient credits").bold().underlined())]
    InsufficientCredits,
    #[error("{}", style("flagged input").bold().underlined())]
    FlaggedInput,
    #[error("{}", style("timed out").bold().underlined())]
    TimedOut,
    #[error("{}", style("rate limited").bold().underlined())]
    RateLimited,
    #[error("{}", style("model down or invalid response").bold().underlined())]
    DownOrInvalid,
    #[error("{}", style("no available providers").bold().underlined())]
    NoProviders,
    #[error("{}", style("unknown error").bold().underlined())]
    Unknown,
}

#[derive(Serialize, Deserialize)]
enum ResponseObject {
    #[serde(rename = "chat.completion")]
    ChatCompletion,
    #[serde(rename = "chat.completion.chunk")]
    ChatCompletionChunk,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Role {
    Assistant,
    System,
    User,
}

pub(crate) fn process_message(input: RandomResult, api: &str, model: &str) -> Result<String> {
    let request_body = Request::new(input, model);
    let agent = Agent::new_with_defaults();
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Processing...");
    spinner.enable_steady_tick(Duration::from_millis(50));

    loop {
        let response = agent
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api))
            .send_json(&request_body)?;

        // unwraps are safe because at this point there is always a response with the expected json
        // schema
        let response: Response = response.into_body().read_json().unwrap();
        let output = &response.choices.first().unwrap().message.content;

        // if the returned response has an empty body, the model is warming up or the system is
        // scaling
        if !output.is_empty() {
            spinner.finish();
            break Ok(output.to_owned());
        }
    }
}

pub(crate) fn response_error(input: Error) -> Error {
    match input.downcast_ref::<ureq::Error>().unwrap() {
        ureq::Error::StatusCode(s) => match s {
            400 => ResponseError::BadRequest.into(),
            401 => ResponseError::InvalidCredentials.into(),
            402 => ResponseError::InsufficientCredits.into(),
            403 => ResponseError::FlaggedInput.into(),
            408 => ResponseError::TimedOut.into(),
            429 => ResponseError::RateLimited.into(),
            502 => ResponseError::DownOrInvalid.into(),
            503 => ResponseError::NoProviders.into(),
            _ => ResponseError::Unknown.into(),
        },
        _ => ResponseError::Unknown.into(),
    }
}
