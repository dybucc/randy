//! This module holds experimental attempts at a TUI for randy.

#![expect(unused, reason = "Temporary allow during development")]

use anyhow::Result;
use console::{style, Key, Term};
use std::fmt::Write as _;

/// This enum holds the information about the types of actions that get triggered with each entry in
/// the menu.
#[derive(PartialEq, Clone)]
pub(crate) enum Action {
    /// This variant is used when the exit button is pressed.
    Finish,
    /// This variant is used when the options page with the model configuration should be shown.
    OptionsPage,
    /// This variant is used when the keybinding wasn't the return key and thus no action should be
    /// triggered.
    Pass,
    /// This variant is used when the game is to be started.
    StartGame,
}

/// This enum holds information about whether one of its variants is currently selected in the menu
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "It's best if the items reflect the actual order they are displayed in the menu."
)]
#[derive(PartialEq, Clone, Copy)]
pub(crate) enum Selected {
    /// This variant is used when the "play" item in the menu is currently selected. It is the item
    /// in the menu that gets selelcted by default once the menu is first loaded.
    Play,
    /// This variant is used when the "options" item in the menu is currently selected.
    Options,
    /// This variant is used when the "exit" item in the menu is currently selected.
    Exit,
}

impl Selected {
    /// This function returns all the enum variants as a vector.
    fn list() -> Vec<Self> {
        vec![Self::Play, Self::Options, Self::Exit]
    }

    /// This function returns the next item in the menu after pressing one of the down arrow or the
    /// up arrow keys.
    fn next(&mut self, key: Key) {
        match self {
            Self::Play => {
                if key == Key::ArrowUp {
                    *self = Self::Exit;
                } else if key == Key::ArrowDown {
                    *self = Self::Options;
                }
            }
            Self::Options => {
                if key == Key::ArrowUp {
                    *self = Self::Play;
                } else if key == Key::ArrowDown {
                    *self = Self::Exit;
                }
            }
            Self::Exit => {
                if key == Key::ArrowUp {
                    *self = Self::Options;
                } else if key == Key::ArrowDown {
                    *self = Self::Play;
                }
            }
        }
    }

    /// This function returns a string representation of the implicit object.
    const fn repr(&self) -> &str {
        match *self {
            Self::Play => "Play",
            Self::Options => "Options",
            Self::Exit => "Exit",
        }
    }
}

/// This function draws the main menu in the screen when the game first loads up.
pub(crate) fn draw_menu(term: &Term, menu: Selected) -> Result<()> {
    let (rows, cols) = term.size();
    let left_half_cols = cols / 2;
    let right_half_cols = cols - left_half_cols;
    let upper_half_rows = rows / 2;
    let lower_half_rows = rows - upper_half_rows;
    let upper_half_list = Selected::list().len() / 2;
    let lower_half_list = Selected::list().len() - upper_half_list;

    for _ in 1..=upper_half_rows as usize - upper_half_list {
        term.write_line("")?;
    }

    for var in Selected::list() {
        let item = var.repr();
        let left_half_item = item.len() / 2;
        let right_half_item = item.len() - left_half_item;
        let mut output = String::new();

        for _ in 1..=left_half_cols as usize - (left_half_item + 3) {
            output.push(' ');
        }

        if var == menu {
            write!(
                output,
                "{}",
                style(format!("   {item}   ")).bold().on_cyan()
            )?;
        } else {
            write!(output, "   {item}   ",)?;
        }

        for _ in 1..=right_half_cols as usize - (right_half_item + 3) {
            output.push(' ');
        }

        term.write_line(&output);
    }

    for _ in 1..=lower_half_rows as usize - lower_half_list {
        term.write_line("")?;
    }

    Ok(())
}

/// This function reads in a key and redraws the menu to select the option corresponding with the
/// arrow key movement.
pub(crate) fn nav(term: &Term, menu: &mut Selected) -> Result<Action> {
    let input = term.read_key()?;

    // check if the input is the enter key; if it is, match with the current contents of the
    // select() function; if it is not, assign to the menu parameter with the next() method
    if input == Key::Enter {
        match menu {
            Selected::Play => return Ok(Action::StartGame),
            Selected::Options => return Ok(Action::OptionsPage),
            Selected::Exit => return Ok(Action::Finish),
        }
    }
    menu.next(input);

    term.clear_screen()?;
    draw_menu(term, output)?;

    Ok(output)
}

/// This function returns the action bound to the specific menu item selected with the return key.
pub(crate) fn select(term: &Term, menu: Selected) -> Result<Action> {
    let input = term.read_key()?;

    if input == Key::Enter {
        match menu {
            Selected::Play => Ok(Action::StartGame),
            Selected::Options => Ok(Action::OptionsPage),
            Selected::Exit => Ok(Action::Finish),
        }
    } else {
        Ok(Action::Pass)
    }
}
