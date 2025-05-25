//! This module contains experimental support for main menu rendering.

use console::Key;

use crate::frame::Selected;

/// This enum holds information about whether one of its variants is currently selected in the menu
#[expect(
    clippy::arbitrary_source_item_ordering,
    reason = "It's best if the items reflect the actual order they are displayed in the menu."
)]
#[derive(PartialEq)]
pub(crate) enum MainMenu {
    /// This variant is used when the "play" item in the menu is currently selected. It is the item
    /// in the menu that gets selelcted by default once the menu is first loaded.
    Play,
    /// This variant is used when the "options" item in the menu is currently selected.
    Options,
    /// This variant is used when the "exit" item in the menu is currently selected.
    Exit,
}

impl Selected for MainMenu {
    type Action = MainMenuAction;

    fn action(&self) -> Self::Action {
        match *self {
            Self::Play => MainMenuAction::StartGame,
            Self::Options => MainMenuAction::OptionsPage,
            Self::Exit => MainMenuAction::Finish,
        }
    }

    /// This function returns all the enum variants as a vector.
    fn list(&self) -> Vec<Self> {
        vec![Self::Play, Self::Options, Self::Exit]
    }

    /// This function returns the next item in the menu after pressing one of the down arrow or the
    /// up arrow keys.
    fn next(&mut self, key: Key) {
        match *self {
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

    fn pass(&self) -> Self::Action {
        MainMenuAction::Pass
    }

    /// This function returns a string representation of the implicit object.
    fn repr(&self) -> &str {
        match *self {
            Self::Play => "Play",
            Self::Options => "Options",
            Self::Exit => "Exit",
        }
    }
}

/// This enum holds the information about the types of actions that get triggered with each entry in
/// the menu.
#[derive(PartialEq)]
pub(crate) enum MainMenuAction {
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
