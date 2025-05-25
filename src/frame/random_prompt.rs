//! This module enables experimental support for a prompt in which to enter two inputs.

use anyhow::Result;
use console::{pad_str, style, Key, Term};
use regex::Regex;

/// This structure holds information about prompts with arbitrary user input.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The fields are best left the same way they appear in the actual frame with the rendered prompt."
)]
#[derive(Default)]
struct Prompt<'contents> {
    /// This field contains the instruction text to be shown to the user.
    text: &'contents str,
    /// This field contains the actual prompt with a string holding the non-buffered input from the
    /// user.
    prompt: String,
}

impl<'contents> Prompt<'contents> {
    /// This function creates a new prompt based on the instruction text and the contents of the prompt.
    fn new(text: &'contents str) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }
}

/// This enumeration contains the possible states of the user selection.
#[derive(Clone, Copy)]
enum Selected {
    /// This variant represents the action of accepting the given inputs.
    Accept,
    /// This variant represents the state of having selected the regular input prompt.
    RandomPrompt,
    /// This variant represents the state of having selected the ranged input prompt.
    RangePrompt,
}

/// This funnction takes two prompts, a selector indicating which of the two is currently selected,
/// and draws on the terminal a frame with the current prompt, as well as the input from the user
/// up to the character they have written out.
fn draw_input_prompt(
    term: &Term,
    prompt_range: &Prompt,
    prompt_random: &Prompt,
    selected: Selected,
    score: u32,
) -> Result<()> {
    let (rows, cols) = term.size();
    let upper_half_fill = rows / 2 - 2;
    let lower_half_fill = rows - rows / 2 - 2;

    for _ in 1..upper_half_fill {
        term.write_line("")?;
    }

    let output1;
    let output2;
    let output3;
    match selected {
        Selected::RandomPrompt => {
            output1 = format!(
                "{}",
                style(if prompt_range.prompt.is_empty() {
                    "()"
                } else {
                    &prompt_range.prompt
                })
                .bold()
            );
            output2 = format!(
                "{}",
                style(if prompt_random.prompt.is_empty() {
                    "()"
                } else {
                    &prompt_random.prompt
                })
                .bold()
                .on_cyan()
            );
            output3 = format!("{}", style("Accept").bold());
        }
        Selected::RangePrompt => {
            output1 = format!(
                "{}",
                style(if prompt_range.prompt.is_empty() {
                    "()"
                } else {
                    &prompt_range.prompt
                })
                .bold()
                .on_cyan()
            );
            output2 = format!(
                "{}",
                style(if prompt_random.prompt.is_empty() {
                    "()"
                } else {
                    &prompt_random.prompt
                })
                .bold()
            );
            output3 = format!("{}", style("Accept").bold());
        }
        Selected::Accept => {
            output1 = format!(
                "{}",
                style(if prompt_range.prompt.is_empty() {
                    "()"
                } else {
                    &prompt_range.prompt
                })
                .bold()
            );
            output2 = format!(
                "{}",
                style(if prompt_random.prompt.is_empty() {
                    "()"
                } else {
                    &prompt_random.prompt
                })
                .bold()
            );
            output3 = format!("{}", style("Accept").bold().on_cyan());
        }
    }

    let output = format!("{}", style(prompt_range.text).bold());
    let output = pad_str(&output, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;

    let output = pad_str(&output1, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;

    let output = format!("{}", style(prompt_random.text).bold());
    let output = pad_str(&output, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;

    let output = pad_str(&output2, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;

    let output = pad_str(&output3, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;

    for _ in 1..lower_half_fill - 2 {
        term.write_line("")?;
    }

    let binding = format!("{}", style(format!("Score {score}")).bold().on_cyan());
    let output = pad_str(&binding, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;

    Ok(())
}

/// This function allows navigation through the input prompts to perform arbitrary input
/// recognition.
pub(crate) fn nav_input_prompt(
    term: &Term,
    validator: (&Regex, &Regex),
    score: u32,
) -> Result<(usize, usize, usize)> {
    let mut prompt_range = Prompt::new("Input a range in the format n..m");
    let mut prompt_random = Prompt::new("Input a random number in the above range");
    let mut selected = Selected::RangePrompt;

    loop {
        term.clear_screen()?;
        draw_input_prompt(term, &prompt_range, &prompt_random, selected, score)?;

        let key = term.read_key()?;
        match selected {
            Selected::RangePrompt if key == Key::Enter => loop {
                let input = term.read_key()?;
                match input {
                    Key::Escape => {
                        if validator.0.is_match(&prompt_range.prompt) {
                            let (start, end) = prompt_range.prompt.split_at(
                                prompt_range.prompt.find("..").expect("pattern not found"),
                            );
                            let end: String = end.chars().rev().collect();
                            let (end, _) = end.split_at(end.find("..").expect("pattern not found"));

                            if let (Ok(_), Ok(_)) = (start.parse::<usize>(), end.parse::<usize>()) {
                                if start < end {
                                    break;
                                }
                            }
                        }
                        prompt_range.prompt.clear();
                    }
                    Key::Backspace => {
                        let _ = prompt_range.prompt.pop();
                    }
                    Key::Char(ch) => prompt_range.prompt.push(ch),
                    _ => {}
                }

                term.clear_screen()?;
                draw_input_prompt(term, &prompt_range, &prompt_random, selected, score)?;
            },
            Selected::RangePrompt if key == Key::ArrowUp => selected = Selected::Accept,
            Selected::RangePrompt if key == Key::ArrowDown => selected = Selected::RandomPrompt,
            Selected::RandomPrompt if key == Key::Enter => loop {
                let input = term.read_key()?;
                match input {
                    Key::Escape => {
                        if validator.1.is_match(&prompt_random.prompt)
                            && !prompt_range.prompt.is_empty()
                        {
                            let (start, end) = prompt_range.prompt.split_at(
                                prompt_range.prompt.find("..").expect("pattern not found"),
                            );
                            let end: String = end.chars().rev().collect();
                            let (end, _) = end.split_at(end.find("..").expect("pattern not found"));
                            let start: usize = start.parse().expect("conversion failed");
                            let end: usize = end.parse().expect("conversion failed");
                            let num: usize =
                                prompt_random.prompt.parse().expect("conversion failed");

                            if num >= start && num <= end {
                                break;
                            }
                        }
                        prompt_random.prompt.clear();
                    }
                    Key::Backspace => {
                        let _ = prompt_random.prompt.pop();
                    }
                    Key::Char(ch) => prompt_random.prompt.push(ch),
                    _ => {}
                }

                term.clear_screen()?;
                draw_input_prompt(term, &prompt_range, &prompt_random, selected, score)?;
            },
            Selected::RandomPrompt if key == Key::ArrowUp => selected = Selected::RangePrompt,
            Selected::RandomPrompt if key == Key::ArrowDown => selected = Selected::Accept,
            Selected::Accept if key == Key::Enter => {
                if !prompt_random.prompt.is_empty() && !prompt_range.prompt.is_empty() {
                    let (start, end) = prompt_range
                        .prompt
                        .split_at(prompt_range.prompt.find("..").expect("pattern not found"));
                    let end: String = end.chars().rev().collect();
                    let (end, _) = end.split_at(end.find("..").expect("pattern not found"));
                    let start: usize = start.parse().expect("conversion failed");
                    let end: usize = end.parse().expect("conversion failed");
                    let num: usize = prompt_random.prompt.parse().expect("conversion failed");

                    break Ok((num, start, end));
                }
            }
            Selected::Accept if key == Key::ArrowUp => selected = Selected::RandomPrompt,
            Selected::Accept if key == Key::ArrowDown => selected = Selected::RangePrompt,
            _ => {}
        }
    }
}
