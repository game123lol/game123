/// Компонент, маркер предмета. Сущность, которая обладает этим компонентом, должна иметь позицию,
/// если она находится на карте, или же она должна находиться в чьём-нибудь инвентаре.
use std::{collections::HashMap, sync::Arc};


use vek::Vec3;


use crate::{
    components::{Name, Position},
    hasher,
    systems::render::Renderable,
    GameHasher, Property,
};

#[derive(Clone)]
pub struct Item {
    pub name: String,
    pub sprite_name: String,
    pub properties: HashMap<String, Property, GameHasher>,
}

impl Item {
    pub fn new(name: String, sprite_name: String) -> Self {
        Self {
            name,
            sprite_name,
            properties: HashMap::with_hasher(hasher()),
        }
    }
    pub fn add_props(&mut self, props: &[(String, Property)]) {
        for (prop_name, prop_val) in props {
            self.properties.insert(prop_name.clone(), prop_val.clone());
        }
    }
    pub fn to_map_entity(
        self,
        pos_x: i32,
        pos_y: i32,
        pos_z: i32,
    ) -> (Renderable, Item, Name, Position) {
        let name = self.name.clone();
        let sprite_name = self.sprite_name.clone();
        (
            Renderable(Arc::from(sprite_name.as_str())),
            self,
            Name(Arc::from(name.as_str())),
            Position(Vec3::new(pos_x, pos_y, pos_z)),
        )
    }
}
