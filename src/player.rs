use std::{collections::HashSet, sync::Arc};

use hecs::{EntityBuilder, World};
use vek::Vec3;

use crate::{
    components::Position,
    items::Item,
    mob::{Inventory, Log},
    need_components,
    systems::{fov_compute::Sight, memory::MapMemory, render::Renderable},
    Mob,
};

/// Компонент, означающий, что сущность с этим компонентом - управляема игроком.
/// Ожидается, что она должна встречаться только один раз в игре.
pub struct Player;

/// Компонент, содержащий историю событий от лица сущности, с которой они происходили.
/// События записаны в текстовом представлении, отделены переносом строки

pub fn new_player() -> EntityBuilder {
    let mut ebuilder = EntityBuilder::new();
    ebuilder.add_bundle((
        Position(Vec3::new(1, 1, 0)),
        Sight(50, HashSet::new()),
        Renderable(Arc::from("person")),
        Player,
        Mob,
        MapMemory::new(),
        Inventory(Vec::new()),
        Log("".to_owned()),
    ));
    ebuilder
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
