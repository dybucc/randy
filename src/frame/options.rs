//! This module holds experimental support for rendering an options menu.

use console::{Key, Term};

use crate::frame::Selected;

/// This enum holds information about whether one of its variants is currently selected in the menu
#[derive(PartialEq)]
pub(crate) enum OptionsMenu {
    /// This variant is used to represent the "model" item in the options menu.
    Model,
    /// This variant is used to represent the option to return back to the frame before the options
    /// menu.
    Return,
}

impl Selected for OptionsMenu {
    type Action = OptionsMenuAction;

    fn action(&self) -> Self::Action {
        match *self {
            Self::Model => OptionsMenuAction::ChangeModel,
            Self::Return => OptionsMenuAction::GoBack,
        }
    }

    /// This function returns all the enum variants as a vector.
    fn list(&self) -> Vec<Self> {
        vec![Self::Model, Self::Return]
    }

    /// This function returns the next item in the menu after pressing one of the down arrow or the
    /// up arrow keys.
    fn next(&mut self, key: Key) {
        match *self {
            Self::Model if key == Key::ArrowUp => {
                *self = Self::Return;
            }
            Self::Model if key == Key::ArrowDown => {
                *self = Self::Return;
            }
            Self::Return if key == Key::ArrowUp => {
                *self = Self::Model;
            }
            Self::Return if key == Key::ArrowUp => {
                *self = Self::Model;
            }
            Self::Return => {}
            Self::Model => {}
        }
    }

    fn pass(&self) -> Self::Action {
        OptionsMenuAction::Pass
    }

    /// This function returns a string representation of the implicit object.
    fn repr(&self) -> &str {
        match *self {
            Self::Model => "Model",
            Self::Return => "Return",
        }
    }
}

/// This enum holds the information about the types of actions that get triggered with each entry in
/// the menu.
#[derive(PartialEq)]
pub(crate) enum OptionsMenuAction {
    /// This variant is used when the user wants to change the model in use.
    ChangeModel,
    /// This variant is used when the user decides to go back from the options menu to the previous
    /// frame.
    GoBack,
    /// This variant is used when the user presses a keybinding that does not trigger any action.
    Pass,
}
