use console::{style, Term};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;

pub(crate) fn take_input(term: &Term) -> usize {
    let input: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("{}", style("Input a number").bold()))
        .interact_text_on(term)
        .unwrap();

    input
}

pub(crate) fn take_ranged_input(term: &Term) -> (usize, usize) {
    // TODO: validate input
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{}",
            style("Input a range in the format n..m (both inclusive)").bold()
        ))
        .interact_text_on(term)
        .unwrap();

    // initially no error handling will be implemented
    let (start, mut end) = input.split_at(input.find("..").unwrap());
    (_, end) = end.split_at(end.find(|v: char| v.is_numeric()).unwrap());

    (start.parse().unwrap(), end.parse().unwrap())
}
