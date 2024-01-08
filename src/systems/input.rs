use tetra::input::get_keys_pressed;

use crate::{Game, PlayerAction, UIState};

pub fn run_input_system(game: &mut Game, ctx: &mut tetra::Context) -> super::Result {
    game.next_action = PlayerAction::Nothing;
    if let Some(key) = get_keys_pressed(ctx).next() {
        match &game.ui_state {
            UIState::No | UIState::Debug => {
                if let Some(val) = game.ui_config.world_keys.get(key) {
                    game.next_action = val.to_owned();
                }
            }
            UIState::Inventory { items: _ } => {
                if let Some(val) = game
                    .ui_config
                    .dialogs_keys
                    .get("inventory")
                    .expect("inventory keymap not set")
                    .get(key)
                {
                    game.next_action = val.to_owned();
                }
            }
        }
    }
    Ok(())
}
