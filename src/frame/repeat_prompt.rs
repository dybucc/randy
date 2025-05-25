//! This module contains experimental support for rendering an input prompt to repeat the game one
//! more time.

use anyhow::Result;
use console::{pad_str, style, Key, Term};

/// This structure holds information about the entire prompt itself.
struct Prompt<'text> {
    /// This field contains the actual input prompt to ask the user out about their decission to
    /// continue playing or not.
    input: Selectable,
    /// This field contains information about whether the "Accept" button or the prompt is currently
    /// selected to then be correctly highlighted.
    selected: PromptSelectable,
    /// This field contains the instruction text to ask the user out if they want to repeat the
    /// game.
    text: &'text str,
}

/// This enumerations represents which of the two elements in the prompt are currently selected
#[derive(PartialEq)]
enum PromptSelectable {
    /// This variant refers to the "Accept" button to press once the prompt has been filled.
    Accept,
    /// This variant refers to the actual input prompt with the option to choose.
    Prompt,
}

/// This enumeration contains information about the option in the prompt that is currently selected.
enum Selectable {
    /// This variant refers to the action of exitting the game altogether.
    No,
    /// This variant refers to the action of repeting another game.
    Yes,
}

/// This function draws a frame with a prompt asking whether the user wants to repeat for another
/// game or not.
fn draw_repeat_prompt(term: &Term, prompt: &Prompt) -> Result<()> {
    let (rows, cols) = term.size();
    let fill = rows / 2 - 1;

    let output1;
    let output2;
    match prompt.selected {
        PromptSelectable::Prompt => {
            output1 = format!(
                "{}",
                style(match prompt.input {
                    Selectable::Yes => "< Yes >",
                    Selectable::No => "< No >",
                })
                .bold()
                .on_cyan()
            );
            output2 = format!("{}", style("Accept").bold());
        }
        PromptSelectable::Accept => {
            output1 = format!(
                "{}",
                style(match prompt.input {
                    Selectable::Yes => "< Yes >",
                    Selectable::No => "< No >",
                })
                .bold()
            );
            output2 = format!("{}", style("Accept").bold().on_cyan());
        }
    }

    for _ in 1..fill {
        term.write_line("")?;
    }

    let output = format!("{}", style(prompt.text.to_owned()).bold());
    let output = pad_str(&output, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;

    let output = pad_str(&output1, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;

    let output = pad_str(&output2, cols as usize, console::Alignment::Center, None);
    term.write_line(&output)?;

    Ok(())
}

/// This function draws a frame in the terminal to draw a prompt that asks the user if they want to
/// repeat for another game.
pub(crate) fn nav_repeat_prompt(term: &Term) -> Result<bool> {
    let mut prompt = Prompt {
        text: "Want to continue for another game?",
        input: Selectable::Yes,
        selected: PromptSelectable::Prompt,
    };

    term.hide_cursor()?;

    loop {
        term.clear_screen()?;
        draw_repeat_prompt(term, &prompt)?;

        let key = term.read_key()?;
        match key {
            Key::ArrowRight | Key::ArrowLeft if prompt.selected == PromptSelectable::Prompt => {
                match prompt.input {
                    Selectable::Yes => prompt.input = Selectable::No,
                    Selectable::No => prompt.input = Selectable::Yes,
                }
            }
            Key::ArrowDown | Key::ArrowUp if prompt.selected == PromptSelectable::Prompt => {
                prompt.selected = PromptSelectable::Accept;
            }
            Key::ArrowDown | Key::ArrowUp if prompt.selected == PromptSelectable::Accept => {
                prompt.selected = PromptSelectable::Prompt;
            }
            Key::Enter if prompt.selected == PromptSelectable::Accept => match prompt.input {
                Selectable::Yes => {
                    term.show_cursor()?;
                    break Ok(true);
                }
                Selectable::No => {
                    term.show_cursor()?;
                    break Ok(false);
                }
            },
            _ => {}
        }
    }
}
