//! This module enables experimental support for basic prompts on fixed frames.

use std::{borrow::Borrow as _, ops::Rem};

use anyhow::Result;
use console::{style, Key, Term};
use serde::Deserialize;

/// This struct holds the single item in the response to the model list request to the OpenRouter
/// API.
#[derive(Deserialize)]
struct Data {
    /// This field contains the unique identifier name of a model.
    id: String,
}

/// This struct holds the response to the model list request to the OpenRouter API.
#[derive(Deserialize)]
struct Response {
    /// This field contains the vector containing information about all models usable through the
    /// API.
    data: Vec<Data>,
}

/// This enum contains the variants for which a prompt may have one element of it or the other
/// selected.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "It's best if the items are kept in the same order as they would appear in the actual menu prompt."
)]
#[derive(Clone)]
enum SelectedItem {
    /// This variant represents that the prompt itself is the one currently selected.
    Selector,
    /// This variant represents the capacity to go back to the previous frame.
    Return,
}

/// This structure links together the information from a prompt and the sliding selector it
/// contains.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "It's best if the prompt comes right after the text."
)]
#[derive(Clone)]
struct SlidingPrompt<'contents> {
    /// This field contains the text giving out the instructions for the prompt.
    text: &'contents str,
    /// This field contains the selector with a single entry per `SlidingPrompt` object.
    selector: String,
    /// This field contains information about whether the `text` field or the `selector` field above
    /// are selected.
    selected: SelectedItem,
}

impl<'contents> SlidingPrompt<'contents> {
    /// This function creates a new sliding prompt with the default selected item set to the text
    /// instructions.
    const fn new(text: &'contents str, selector: String) -> Self {
        Self {
            text,
            selector,
            selected: SelectedItem::Selector,
        }
    }

    /// This function mutates the state of the sliding prompt to alter the currently appearing
    /// selector field. It is thus best used with a single `SlidingPrompt` object, and a collection
    /// of selector items to sort through and quickly change.
    fn switch_selector(&mut self, other: String) {
        self.selector = other;
    }
}

/// This function draws and updates a frame with a prompt and a sliding selector.
fn draw_sliding_prompt(term: &Term, prompt: &SlidingPrompt) -> Result<()> {
    let (rows, cols) = term.size();
    let upper_half_list = rows / 2 - 1;
    let lower_half_list = rows - rows / 2 - 2;

    for _ in 1..upper_half_list {
        term.write_line("")?;
    }

    let text = format!("{}", style(format!("   {}   ", prompt.text)).bold());
    let text = console::pad_str(&text, cols as usize, console::Alignment::Center, None);
    term.write_line(&text)?;

    let ret;
    let selector;
    match prompt.selected {
        SelectedItem::Return => {
            selector = format!("{}", style(format!("   {}   ", prompt.selector)).bold());
            ret = format!("{}", style(format!("   {}   ", "Return")).bold().on_cyan());
        }
        SelectedItem::Selector => {
            selector = format!(
                "{}",
                style(format!("   {}   ", prompt.selector)).bold().on_cyan()
            );
            ret = format!("{}", style(format!("   {}   ", "Return")).bold());
        }
    }

    let selector = console::pad_str(&selector, cols as usize, console::Alignment::Center, None);
    term.write_line(&selector)?;

    let ret = console::pad_str(&ret, cols as usize, console::Alignment::Center, None);
    term.write_line(&ret)?;

    for _ in 1..lower_half_list {
        term.write_line("")?;
    }

    Ok(())
}

/// This function takes a model value, and depending on which model is set, either changes focus
/// from the text prompt to the model or otherwise changes the model to another one. Thus it also
/// makes a request to the OpenRouter API to fetch the model list and display it as a sliding
/// window.
pub(crate) fn nav_sliding_prompt(term: &Term, model: &mut String) -> Result<()> {
    let request: Response = ureq::get("https://openrouter.ai/api/v1/models")
        .call()?
        .into_body()
        .read_json()?;
    let models: Vec<String> = request.data.into_iter().map(|value| value.id).collect();
    let mut prompt = SlidingPrompt::new(
        "Select a model below; use the left and right arrow keys",
        format!("< {model} >"),
    );

    loop {
        draw_sliding_prompt(term, &prompt)?;

        let key = term.read_key()?;
        match key {
            Key::ArrowLeft if matches!(prompt.selected, SelectedItem::Selector) => {
                match models.get(
                    models
                        .iter()
                        .position(|value| value == model)
                        .expect("model not found")
                        .wrapping_sub(1),
                ) {
                    None => {
                        let last = models.last().expect("empty model list");
                        prompt.switch_selector(format!("< {} >", last));
                        model.clone_from(last);
                    }
                    Some(mo) => {
                        prompt.switch_selector(format!("< {} >", mo));
                        model.clone_from(mo);
                    }
                }
            }
            Key::ArrowRight if matches!(prompt.selected, SelectedItem::Selector) => {
                match models.get(
                    models
                        .iter()
                        .position(|value| value == model)
                        .expect("model not found")
                        + 1,
                ) {
                    None => {
                        let first = models.first().expect("empty model list");
                        prompt.switch_selector(format!("< {} >", first));
                        model.clone_from(first);
                    }
                    Some(mo) => {
                        prompt.switch_selector(format!("< {} >", mo));
                        model.clone_from(mo);
                    }
                }
            }
            Key::ArrowUp if matches!(prompt.selected, SelectedItem::Selector) => {
                prompt.selected = SelectedItem::Return;
            }
            Key::ArrowDown if matches!(prompt.selected, SelectedItem::Selector) => {
                prompt.selected = SelectedItem::Return;
            }
            Key::ArrowUp if matches!(prompt.selected, SelectedItem::Return) => {
                prompt.selected = SelectedItem::Selector;
            }
            Key::ArrowDown if matches!(prompt.selected, SelectedItem::Return) => {
                prompt.selected = SelectedItem::Selector;
            }
            Key::Enter if matches!(prompt.selected, SelectedItem::Return) => break,
            _ => {}
        }
    }

    Ok(())
}
