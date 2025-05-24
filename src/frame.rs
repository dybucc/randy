//! This module holds experimental attempts at a TUI for randy.

#![expect(unused, reason = "Temporary allow during development")]

pub(crate) mod main_menu;
pub(crate) mod options;
pub(crate) mod prompt;
pub(crate) mod random_prompt;

use anyhow::Result;
use console::{style, Key, Term};
use std::fmt::Write as _;

/// This trait implements methods for menus with selectable items.
pub(crate) trait Selected
where
    Self: Sized + PartialEq,
{
    /// This type relates whatever object implements the trait with another object that defines the
    /// action associated with the menu.
    type Action;

    /// This function returns the non-noop actions that the object implementing the trait can
    /// trigger.
    fn action(&self) -> Self::Action;
    /// This function returns a list of the items contained in the object that implements the trait.
    fn list(&self) -> Vec<Self>;
    /// This function mutates the state of the object implementing the trait depending on an input
    /// key.
    fn next(&mut self, key: Key);
    /// This function returns the noop action that the object implementing the trait is forced to
    /// trigger when no key sequence selects an item with an associated action.
    fn pass(&self) -> Self::Action;
    /// This function returns the string representation of an element of the current object
    /// implementing the trait.
    fn repr(&self) -> &str;
}

/// This function draws a menu in the screen whenever the game requests a menu to load up.
pub(crate) fn draw_menu<T>(term: &Term, menu: &T) -> Result<()>
where
    T: Selected,
{
    let (rows, cols) = term.size();
    let left_half_cols = cols / 2;
    let right_half_cols = cols - left_half_cols;
    let upper_half_rows = rows / 2;
    let lower_half_rows = rows - upper_half_rows;
    let upper_half_list = menu.list().len() / 2;
    let lower_half_list = menu.list().len() - upper_half_list;

    term.clear_screen()?;

    for _ in 1..=upper_half_rows as usize - upper_half_list {
        term.write_line("")?;
    }

    for var in menu.list() {
        let item = var.repr();

        let content = if var == *menu {
            format!("{}", style(item).bold().on_cyan())
        } else {
            format!("{}", style(item).bold())
        };

        let output = console::pad_str(&content, cols as usize, console::Alignment::Center, None);
        term.write_line(&output);
    }

    for _ in 1..lower_half_rows as usize - lower_half_list {
        term.write_line("")?;
    }

    Ok(())
}

/// This function reads in a key and redraws a menu to select the option corresponding with the
/// arrow key movement.
pub(crate) fn nav_menu<T>(term: &Term, menu: &mut T) -> Result<T::Action>
where
    T: Selected,
{
    let input = term.read_key()?;

    if input == Key::Enter {
        return Ok(menu.action());
    }
    menu.next(input);

    term.clear_screen()?;
    draw_menu(term, menu)?;

    Ok(menu.pass())
}
