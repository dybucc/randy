//! This module takes on the role of contacting the OpenRouter API and make requests to it. The main
//! part of the module consists of different structures to serialize the information about the
//! requests that is required and expected from the model.
//!
//! There are also some functions to actually make the requests, and process the possible errors
//! that might come from the request.

use std::{sync::LazyLock, time::Duration};

use anyhow::{Error, Result};
use console::style;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::game::RandomResult;

/// This static variable contains the text to be fed to the LLM in the request to the OpenRouter
/// API. It was decided to be made a lazy static because the string is fairly long, and it's
/// preferable for it to be initialized once it is required.
static LLM_INPUT: LazyLock<&str> = LazyLock::new(|| {
    "You will answer only to \"Correct\" or \"Incorrect.\" These correspond to either a\
notification that a user got a number right in a number guessing game or not, respectively. Your\
task is to, depending on whether you were notified they got it right, or not, to return a\
cowboy-like answer to the user. Make it a short text. Include just your answer and nothing more.\
Don't include emoji or otherwise non-verbal content."
});

/// This enum serves as a way of extending the possible errors from the default requests, so as to
/// smooth the experience of the user.
#[derive(thiserror::Error, Debug, PartialEq)]
enum ExtraError {
    /// This variant refers to a manual time out that has been, for now, hardcoded to allow exitting
    /// if the request has no content for more than 10 requests.
    #[error("{}", style("timed out after multiple requests").bold())]
    TimedOut,
}

/// This structure contains one of the fields sent to the POST request to the OpenRouter API for
/// chat completion.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The JSON schema needs the fields to be in this order."
)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Message {
    /// This field is one of the required fields in the request to the OpenRouter API.
    role: Role,
    /// This field is one of the required fields in the request to the OpenRouter API.
    content: String,
}

impl Message {
    /// This function returns a new `Message` object to be used with the request builder in the
    /// function of the same name for the `Request` structure.
    fn new(role: Role, content: &str) -> Self {
        Self {
            role,
            content: content.to_owned(),
        }
    }
}

/// This structure holds the main source of information about the request to the OpenRouter API for
/// chat completion. It contains as well a builder function of the request with the predefined
/// defaults required by this program.
#[derive(Serialize, Debug, PartialEq)]
struct Request {
    /// This field contains information about the messages to be sent to the LLM.
    messages: Vec<Message>,
    /// This field contains information about the model to use in the request.
    model: String,
}

impl Request {
    /// This function serves as a request builder to be sent to the LLM. It takes up the input as a
    /// variant of the result obtained by the user, and creates a slightly different request
    /// depending on that. It also takes up the model name to be used by the request, which is
    /// either chosen by the user or defaulted to a specific free model in another part of the
    /// program.
    fn new(input: RandomResult, model: &str) -> Self {
        match input {
            RandomResult::Correct => Self {
                messages: vec![
                    Message::new(Role::System, *LLM_INPUT),
                    Message::new(Role::User, "Correct"),
                ],
                model: model.to_owned(),
            },
            RandomResult::Incorrect => Self {
                messages: vec![
                    Message::new(Role::System, *LLM_INPUT),
                    Message::new(Role::User, "Incorrect"),
                ],
                model: model.to_owned(),
            },
        }
    }
}

/// This structure serves as the container for the response obtained by the LLM with OpenRouter's
/// API's POST request for chat completion.
#[derive(Serialize, Deserialize)]
struct Response {
    /// This field contains the responses returned by the LLM, which in the case of purely
    /// text-based queries, is made up of a single element.
    choices: Vec<ResponseChoices>,
}

/// This structure serves as part of the request JSON sent to the OpenRouter API containing the
/// actual messages behind the LLM response, as well as some other details particular to the
/// response.
#[derive(Serialize, Deserialize)]
struct ResponseChoices {
    /// This field contains the `Message` object, with information about both the role of the LLM
    /// in the response, and the response itself.
    message: Message,
}

/// This enum holds information about all the errors documented in the OpenRouter documentation
/// site for any one of their API requests.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "It's easier to maintain if the errors are in the same order as the ones specified in the OpenRouter docs."
)]
#[derive(thiserror::Error, Debug)]
enum ResponseError {
    /// This error reports whether the request was somehow incorrect, corrupted or it simply failed.
    #[error("{}", style("bad request").bold().underlined())]
    BadRequest,
    /// This error reports whether the request was made with invalid credentials, i.e. the request's
    /// API key was not valid, as that is the only form of authentication used.
    #[error("{}", style("invalid credentials").bold().underlined())]
    InvalidCredentials,
    /// This error reports that the amount of credits in the OpenRouter user account associated
    /// with the request's API key isn't enough to actually use the LLM of choice. This error should
    /// only take place if either the credits are negative, or otherwise if the chosen model is not
    /// free.
    #[error("{}", style("insufficient credits").bold().underlined())]
    InsufficientCredits,
    /// This error reports that the input in the request was flagged as inappropiate and thus also
    /// reveals the model contains filtering policies. This error shouldn't ever happen, considering
    /// the request's content is defined by the program and the user has no take in it.
    #[error("{}", style("flagged input").bold().underlined())]
    FlaggedInput,
    /// This error reports that the request timed out.
    #[error("{}", style("timed out").bold().underlined())]
    TimedOut,
    /// This error reports that the request was rate limited, generally because a free model is
    /// being used, and the amount of request per minute or per day has been surpassed.
    #[error("{}", style("rate limited").bold().underlined())]
    RateLimited,
    /// This error reports whether the model is down for maintenance or otherwise produced an
    /// invalid response.
    #[error("{}", style("model down or invalid response").bold().underlined())]
    DownOrInvalid,
    /// This error reports that there are no available providers for the requested model. This error
    /// is rare, as there are generally at least 1-2 providers even for the least-used models.
    #[error("{}", style("no available providers").bold().underlined())]
    NoProviders,
    /// This error reports that an unknown error has taken place. An unknown error is one which is
    /// not any of the above variants.
    #[error("{}", style("unknown error").bold().underlined())]
    Unknown,
}

/// This enum holds the different roles the LLM or the user can take on during a chat completion
/// request.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
enum Role {
    /// This variant contains the assistant role used by the LLM on text-based chat completion
    /// requests to answer back.
    Assistant,
    /// This variant contains the system role used by the user to specify a specific expected
    /// behavior from the LLM.
    System,
    /// This variant contains the user role used by the user to query regular prompts to the LLM.
    User,
}

/// This function has the role of processing the result of the current game, and making a request
/// to the OpenRouter API depending on whether they won or lost, so as to return the response of the
/// LLM.
pub(crate) fn process_message(input: RandomResult, api_key: &str, model: &str) -> Result<String> {
    let request_body = Request::new(input, model);
    let agent = Agent::new_with_defaults();
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Processing...");
    spinner.enable_steady_tick(Duration::from_millis(50));
    let mut repeated = 0;

    loop {
        let response = agent
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .send_json(&request_body)?;

        // unwraps are safe because at this point there is always a response with the expected json
        // schema
        let response: Response = response.into_body().read_json().expect("response failed");
        let output = &response
            .choices
            .first()
            .expect("no elements were found")
            .message
            .content;

        // if the returned response has an empty body, the model is warming up or the system is
        // scaling
        if !output.is_empty() {
            spinner.finish_and_clear();
            break Ok(output.to_owned());
        } else if repeated > 10 {
            break Err(ExtraError::TimedOut.into());
        }

        repeated += 1;
    }
}

/// This function handles errors that take place during the request retrieval step. This is done
/// solely by means of checking the status code returned by the underlying ureq error. This also
/// means whatever was carried in the body of the faulty response is completely discarded.
pub(crate) fn response_error(input: Error) -> Error {
    if *input
        .downcast_ref::<ExtraError>()
        .expect("no underlying error found")
        == ExtraError::TimedOut
    {
        return input.downcast().expect("no underlying error found");
    }

    match *input
        .downcast_ref::<ureq::Error>()
        .expect("no underlying error found")
    {
        ureq::Error::StatusCode(status) => match status {
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

#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "Tests are best left to the end of source files."
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_is_correct() {
        let input = (Role::Assistant, "assistant");
        let expect = Message {
            role: Role::Assistant,
            content: "assistant".to_owned(),
        };
        let actual = Message::new(input.0, input.1);

        assert_eq!(expect, actual);
    }

    #[test]
    fn correct_request_is_right() {
        let input = (RandomResult::Correct, "deepseek");
        let expect = Request {
            messages: vec![
                Message::new(Role::System, *LLM_INPUT),
                Message::new(Role::User, "Correct"),
            ],
            model: "deepseek".to_owned(),
        };
        let actual = Request::new(input.0, input.1);

        assert_eq!(expect, actual);
    }

    #[test]
    fn incorrect_request_is_right() {
        let input = (RandomResult::Incorrect, "deepseek");
        let expect = Request {
            messages: vec![
                Message::new(Role::System, *LLM_INPUT),
                Message::new(Role::User, "Incorrect"),
            ],
            model: "deepseek".to_owned(),
        };
        let actual = Request::new(input.0, input.1);

        assert_eq!(expect, actual);
    }

    #[test]
    #[should_panic = "invalid credentials"]
    // The below function can't deterministically check for the sucess of the function, but it can
    // check if the function either returns an error or otherwise shows a non-empty string.
    fn incorrect_api_key_is_detected() {
        // api key is secure; it's set to have 0 credits available in my openrouter page
        let input = (
            RandomResult::Correct,
            "sk",
            "deepseek/deepseek-chat-v3-0324:free",
        );

        match process_message(input.0, input.1, input.2) {
            Ok(_) => (),
            Err(error) => {
                if matches!(
                    error.downcast::<ureq::Error>().expect("not a ureq error"),
                    ureq::Error::StatusCode(401)
                ) {
                    panic!("invalid credentials")
                }
            }
        }
    }
}
