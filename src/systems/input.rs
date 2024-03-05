use std::collections::HashMap;
use thiserror::Error;

use tetra::input::{get_keys_pressed, Key};

use crate::{Game, PlayerAction, UIState};

#[derive(Error, Debug)]
pub enum InputSystemError {
    #[error("Keymap {0} is not set")]
    KeyMapNotSet(String),
}

pub type InputSystemResult<T> = Result<T, InputSystemError>;

fn get_dialog<'a>(
    game: &'a Game,
    dialog_name: &str,
) -> InputSystemResult<&'a HashMap<Key, PlayerAction>> {
    game.ui_config
        .dialogs_keys
        .get(dialog_name)
        .ok_or(InputSystemError::KeyMapNotSet(dialog_name.to_string()))
}

pub fn run_input_system(game: &mut Game, ctx: &mut tetra::Context) -> InputSystemResult<()> {
    game.next_action = PlayerAction::Nothing;
    if let Some(key) = get_keys_pressed(ctx).next() {
        match &game.ui_state {
            UIState::No | UIState::Debug => {
                if let Some(val) = game.ui_config.world_keys.get(key) {
                    game.next_action = val.to_owned();
                }
            }
            UIState::Inventory { items: _ } => {
                if let Some(val) = get_dialog(game, "inventory")?.get(key) {
                    game.next_action = val.to_owned();
                }
            }
            UIState::Log { text } => {
                if let Some(val) = get_dialog(game, "log")?.get(key) {
                    game.next_action = val.to_owned();
                    //TODO: Код всё равно дублируется
                }
            }
        }
    }
    Ok(())
}
