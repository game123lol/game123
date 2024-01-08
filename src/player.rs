use std::collections::BTreeSet;

use hecs::{Entity, World};
use tetra::math::Vec2;

use crate::{
    components::{Inventory, MapMemory, Mob, Player, Position, Renderable, Sight},
    need_components,
};

type PlayerType = (
    Position,
    Sight,
    Renderable,
    Player,
    Mob,
    MapMemory,
    Inventory,
);

pub const fn new_player() -> PlayerType {
    (
        Position(Vec2::new(1, 1)),
        Sight(50, BTreeSet::new()),
        Renderable("person"),
        Player,
        Mob,
        MapMemory::new(),
        Inventory(Vec::new()),
    )
}

pub fn get_player_items(world: &World) -> anyhow::Result<Vec<Entity>> {
    let mut binding = world.query::<(&Player, &Inventory)>();
    let (_, (_, Inventory(vec))) = binding.into_iter().next().ok_or(need_components!(
        Function_get_player_items,
        Player,
        Inventory
    ))?;

    Ok(vec.to_owned())
}
