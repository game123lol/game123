use macroquad::prelude::{clear_input_queue, get_char_pressed};
use std::collections::HashMap;
use thiserror::Error;

use crate::{Game, GameHasher, PlayerAction, UIAction, UIState};

#[derive(Error, Debug)]
pub enum InputSystemError {
    #[error("Keymap {0} is not set")]
    KeyMapNotSet(String),
}

pub type InputSystemResult<T> = Result<T, InputSystemError>;

fn get_dialog<'a>(
    game: &'a Game,
    dialog_name: &str,
) -> InputSystemResult<&'a HashMap<char, UIAction, GameHasher>> {
    game.ui_config
        .dialogs_keys
        .get(dialog_name)
        .ok_or(InputSystemError::KeyMapNotSet(dialog_name.to_string()))
}

pub fn run_input_system(game: &mut Game) -> InputSystemResult<()> {
    game.next_action = PlayerAction::Nothing;
    for key in get_char_pressed().into_iter() {
        match &game.ui {
            None => {
                if let Some(val) = game.ui_config.world_keys.get(&key) {
                    game.next_action = PlayerAction::Player(val.to_owned());
                }
            }
            Some(ui) => {
                let dialog_name = match ui {
                    UIState::Inventory { .. } => "inventory",
                    UIState::Log { .. } => "log",
                };
                if let Some(val) = get_dialog(game, dialog_name)?.get(&key) {
                    game.next_action = PlayerAction::UI(*val);
                }
            }
        }
    }
    clear_input_queue();
    Ok(())
}
