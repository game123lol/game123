/// Компонент, маркер предмета. Сущность, которая обладает этим компонентом, должна иметь позицию,
/// если она находится на карте, или же она должна находиться в чьём-нибудь инвентаре.
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::Deserialize;

use crate::{
    components::{Name, Position},
    hasher, search,
    systems::render::Renderable,
    GameHasher, Property,
};

macro_rules! get_vals {
    ($map:expr, $($key:expr),*) => {
        'get_vals: {
            let result = Ok(($(
                match $map.get($key) {
                    Some(value) => value.clone(),
                    None => { break 'get_vals Err(format!("Ключ '{}' не найден", $key)) },
                }
            ),*));
            result
        }
    };
}

// Пример взаимодействия предмета с чем-либо:
// Топор находится в огне. У топора два материала: дерево и сталь
// по идее рукоять топора должна сгореть
// но чё получится в итоге

// pub struct Material {}

#[derive(Clone, Debug, Deserialize)]
pub struct Item {
    pub name: String,
    pub sprite_name: String,
    pub properties: HashMap<String, Property, GameHasher>,
    pub flags: HashSet<String>,
}

impl Item {
    pub fn new(name: String, sprite_name: String) -> Self {
        Self {
            name,
            sprite_name,
            properties: HashMap::with_hasher(hasher()),
            flags: HashSet::new(),
        }
    }
    #[allow(dead_code)]
    pub fn add_props(&mut self, props: &[(String, Property)]) {
        for (prop_name, prop_val) in props {
            self.properties.insert(prop_name.clone(), prop_val.clone());
        }
    }
    pub fn into_map_entity(self, pos: &Position) -> (Renderable, Item, Name, Position) {
        let name = self.name.clone();
        let sprite_name = self.sprite_name.clone();
        (
            Renderable(Arc::from(sprite_name.as_str())),
            self,
            Name(Arc::from(name.as_str())),
            *pos,
        )
    }
    /// Функция, которая вне зависимости от шанса попадания, высчитывает
    /// прокалывание одним предметом другой предмет
    pub fn apply_pierce(
        &mut self,
        target_item: &Item,
        strength: f64,
        target_part: (&str, &str),
    ) -> Result<(), String> {
        // Вроде бы эта куча проверок не должна быть на каждый раз, и надо бы как-нибудь вынести всё это в валидатор
        // Я хуй знает как, сделаю потом
        // TODO: сделать валидацию
        if !self.flags.contains("can_pierce") {
            todo!()
        }
        let layers = search(&target_item.properties, &["armor", "layers"])?
            .to_list()
            .ok_or(format!(
                "item {}: layers in is not a list",
                target_item.name
            ))?;
        for layer in layers {
            let map = layer.to_map().ok_or("layer is not a map")?;
            if search(map, &[target_part.0, target_part.1]).is_err() {
                break;
            }
            // я устал
            let piercing_resistance = map.get("cut").unwrap().to_float().unwrap();
            let sharpness = self
                .properties
                .get("sharpness")
                .unwrap()
                .to_float()
                .unwrap();
            let pressure = strength * 1000000. / (sharpness * sharpness);
        }
        Ok(())
    }
    pub fn apply_ballistic_penetration(&mut self, target: &Item, velocity: i32) {}
    pub fn apply_blunt(&mut self, target: &Item, velocity: i32) {}
}
