use std::{collections::HashSet, sync::Arc};

use hecs::{EntityBuilder, With, World};
use vek::Vec3;

use crate::{
    components::Position,
    hasher,
    inventory::Inventory,
    mob::Log,
    need_components,
    resources::Resources,
    systems::{fov_compute::Sight, memory::MapMemory, render::Renderable},
    Mob,
};

/// Компонент, означающий, что сущность с этим компонентом - управляема игроком.
/// Ожидается, что она должна встречаться только один раз в игре.
#[derive(Debug)]
pub struct Player;

/// Компонент, содержащий историю событий от лица сущности, с которой они происходили.
/// События записаны в текстовом представлении, отделены переносом строки
pub fn new_player(resources: &Resources) -> EntityBuilder {
    let mut ebuilder = EntityBuilder::new();
    ebuilder.add_bundle((
        Position(Vec3::new(1, 1, 0)),
        Sight(40, HashSet::with_hasher(hasher())),
        Renderable(Arc::from("person")),
        Player,
        Mob,
        MapMemory::new(),
        Inventory::new(),
        Log("".to_owned()),
    ));
    let body = resources.template_body("human");
    ebuilder.add(body);
    ebuilder
}

pub fn get_player_inventory(world: &World) -> anyhow::Result<Inventory> {
    let mut binding = world.query::<With<&Inventory, &Player>>();
    let (_, inventory) = binding.into_iter().next().ok_or(need_components!(
        Function_get_player_items,
        Player,
        Inventory
    ))?;

    Ok(inventory.clone())
}
