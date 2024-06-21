use std::sync::Arc;

use hecs::EntityBuilder;
use serde::Deserialize;
use vek::Vec3;

use crate::{
    components::Position,
    items::Item,
    systems::{memory::MapMemory, pathfinding::Pathfinder, render::Renderable},
    Mob,
};

/// Компонент, содержащий историю событий от лица сущности, с которой они происходили.
/// События записаны в текстовом представлении, отделены переносом строки
pub struct Log(pub String);

pub struct Inventory(pub Vec<Item>);

impl Log {
    pub fn write(&mut self, event: &str) {
        self.0.push_str((event.to_owned() + "\n").as_str());
    }
}

pub fn new_mob(pos: Vec3<i32>) -> EntityBuilder {
    let mut ebuilder = EntityBuilder::new();
    ebuilder.add_bundle((
        Position(pos),
        Renderable(Arc::from("killer")),
        Mob,
        // DummyHealth(10),
        Pathfinder,
        MapMemory::new(),
        Inventory(Vec::new()),
        Log("".to_owned()),
    ));
    ebuilder
}
