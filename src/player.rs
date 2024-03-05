use std::{collections::BTreeSet, sync::Arc};

use hecs::World;
use tetra::math::Vec2;

use crate::{
    components::Position,
    items::Item,
    need_components,
    systems::{fov_compute::Sight, memory::MapMemory, render::Renderable},
    Mob,
};

/// Компонент, означающий, что сущность с этим компонентом - управляема игроком.
/// Ожидается, что она должна встречаться только один раз в игре.
pub struct Player;

/// Компонент, содержащий историю событий от лица сущности, с которой они происходили.
/// События записаны в текстовом представлении, отделены переносом строки
pub struct Log(pub String);

impl Log {
    pub fn write(&mut self, event: &str) {
        self.0.push_str(event);
    }
}

pub struct Inventory(pub Vec<Item>);

type PlayerType = (
    Position,
    Sight,
    Renderable,
    Player,
    Mob,
    MapMemory,
    Inventory,
    Log,
);

pub fn new_player() -> PlayerType {
    (
        Position(Vec2::new(1, 1)),
        Sight(50, BTreeSet::new()),
        Renderable(Arc::from("person")),
        Player,
        Mob,
        MapMemory::new(),
        Inventory(Vec::new()),
        Log("".to_owned()),
    )
}

pub fn get_player_items(world: &World) -> anyhow::Result<Vec<Item>> {
    let mut binding = world.query::<(&Player, &Inventory)>();
    let (_, (_, Inventory(vec))) = binding.into_iter().next().ok_or(need_components!(
        Function_get_player_items,
        Player,
        Inventory
    ))?;

    Ok(vec.to_owned())
}
