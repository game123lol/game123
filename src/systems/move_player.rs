use hecs::World;
use tetra::{
    input::{get_keys_down, Key},
    math::Vec2,
    Context,
};

use crate::{Player, Position};

pub fn move_player_system(world: &mut World, ctx: &Context) {
    let (_, (_, Position(pos))) = world
        .query_mut::<(&Player, &mut Position)>()
        .into_iter()
        .next()
        .unwrap();
    for key in get_keys_down(&ctx) {
        match key {
            Key::W => {
                *pos += Vec2::new(0.0, -1.0);
            }
            Key::S => {
                *pos += Vec2::new(0.0, 1.0);
            }
            Key::A => {
                *pos += Vec2::new(-1.0, 0.0);
            }
            Key::D => {
                *pos += Vec2::new(1.0, 0.0);
            }
            _ => {}
        }
    }
}
