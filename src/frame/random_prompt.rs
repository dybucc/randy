//! This module enables experimental support for a prompt in which to enter two inputs.

use anyhow::Result;
use console::{pad_str, style, Term};

/// This structure holds information about prompts with arbitrary user input.
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "The fields are best left the same way they appear in the actual frame with the rendered prompt."
)]
struct Prompt<'contents> {
    /// This field contains the instruction text to be shown to the user.
    text: &'contents str,
    /// This field contains the actual prompt with a string holding the non-buffered input from the
    /// user.
    prompt: String,
}

/// This enumeration contains the possible states of the user selection.
enum Selected {
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
) -> Result<()> {
    let (rows, cols) = term.size();
    let upper_half_fill = rows / 2 - 2;
    let lower_half_fill = rows - rows / 2 - 2;

    for _ in 1..upper_half_fill {
        term.write_line("")?;
    }

    let output = format!("{}", style("Input a range").bold());
    let output = pad_str(&output, cols as usize, console::Alignment::Center, None);
    term.write_line(&output);

    let prompt1;
    let prompt2;
    match selected {
        Selected::RandomPrompt => prompt1 = format!(""),
        Selected::RangePrompt => {
            todo!()
        }
    }

    Ok(())
}
