//! This module contains all functions related to taking input from the user. They all use the
//! `dialoguer` crate to process the input, and they all check for input validation.
//!
//! Specifically, the two available functions so far take input for the user's guess, and take a
//! range of inputs from which to source the random number.

use anyhow::Result;
use console::{style, Term};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use regex::Regex;

/// This function is in charge of taking the input for the number guess made by the user after
/// taking the range in which they want to play.
pub(crate) fn take_input(term: &Term, range: &(usize, usize)) -> Result<usize> {
    let input: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{}", style("Input a number").bold()))
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.as_bytes().iter().all(|c| c.is_ascii_digit()) {
                // unwrap is safe; at this point, the string is knwown to be solely made out of
                // digits
                let num: usize = input.parse().unwrap();

                if num >= range.0 && num <= range.1 {
                    return Ok(());
                }

                Err("The given input is not within the provided range")
            } else {
                Err("The input should be made up of numbers only")
            }
        })
        .interact_text_on(term)?
        .parse()
        // unwrap is safe; the input was validated with dialoguer's validate_with() method
        .unwrap();

    Ok(input)
}

/// This function is in charge of taking a ranged input of values from the user to pick a number to
/// guess. These values will serve as the bounds of the game and the one number that the user will
/// later try to guess will be found within this range.
pub(crate) fn take_ranged_input(term: &Term, re: Regex) -> Result<(usize, usize)> {
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{}",
            style("Input a range in the format n..m (both inclusive)").bold()
        ))
        .validate_with(|s: &String| -> Result<(), &str> {
            if re.is_match(s) {
                // unwrap is safe; the two dots are part of the regex that must pass before this is
                // checked
                let (start, end) = s.split_at(s.find("..").unwrap());
                let mut end: String = end.chars().rev().collect();
                end.truncate(1);
                let start = start.parse::<usize>();
                let end = end.parse::<usize>();

                match (start, end) {
                    (Ok(b), Ok(e)) if b < e => return Ok(()),
                    (Ok(_), Ok(_)) => return Err("Invalid input; start must be smaller than end"),
                    _ => return Err("Invalid input; check bounds with usize"),
                }
            }
            Err("Invalid input; input can only be numeric")
        })
        .interact_text_on(term)?;

    let (start, mut end) = input.split_at(input.find("..").unwrap());
    (_, end) = end.split_at(end.find(|v: char| v.is_numeric()).unwrap());

    Ok((start.parse().unwrap(), end.parse().unwrap()))
}
