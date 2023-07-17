use std::io::Error;

use hecs::World;
use tetra::{
    input::{get_keys_down, Key},
    Context, TetraError,
};

use crate::{
    entities::{Item, Player, Position},
    map::Map,
};

pub fn run_pickup_system(world: &World, ctx: &mut Context) {
    let mut binding = world.query::<(&Player, &Position)>();
    let (_, (_, Position(pos))) = binding.into_iter().next().unwrap();
    let mut binding = world.query::<(&Map,)>();

    let item = world.query::<(&Item, &Position)>();

    let (_, (map,)) = binding.into_iter().next().unwrap();
    for key in get_keys_down(&ctx) {
        match key {
            Key::G => {}
            _ => {}
        }
    }
}
