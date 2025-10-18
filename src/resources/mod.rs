pub mod assets;
pub mod templates;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use hecs::EntityBuilder;

use serde::Deserialize;

use crate::{
    body::Body,
    hasher,
    inventory::Inventory,
    items::Item,
    mob::Log,
    resources::templates::body::BodyTemplates,
    systems::{fov_compute::Sight, memory::MapMemory, pathfinding::Pathfinder, render::Renderable},
    Mob,
};
use assets::Assets;
use templates::entity::EntityTemplates;
use templates::item::ItemTemplates;

pub struct Resources {
    pub assets: Assets,
    pub entity_templates: EntityTemplates,
    pub body_templates: BodyTemplates,
    pub item_templates: ItemTemplates,
}

#[derive(Deserialize, Debug)]
struct ComponentTemplates {
    body: Option<String>,
    sprite: Option<String>,
    sight: Option<u32>,
    behaviors: Option<HashSet<String>>,
    #[serde(flatten)]
    dynamic_comps: HashMap<String, String>,
}

impl ComponentTemplates {
    pub fn to_entity_builder(&self, body_templates: &BodyTemplates) -> EntityBuilder {
        let mut eb = EntityBuilder::new();
        if let Some(body_name) = &self.body {
            eb.add(body_templates.template(body_name));
        }
        if let Some(sprite_name) = &self.sprite {
            eb.add(Renderable(sprite_name.to_owned().into()));
        }
        if let Some(sight) = &self.sight {
            eb.add(Sight(*sight, HashSet::with_hasher(hasher())));
        }
        if let Some(behaviors) = &self.behaviors {
            for behavior in behaviors {
                match behavior.as_str() {
                    "mob" => {
                        eb.add(Mob);
                    }
                    "pathfinder" => {
                        eb.add(Pathfinder);
                    }
                    "map_memory" => {
                        eb.add(MapMemory::new());
                    }
                    "inventory" => {
                        eb.add(Inventory::new());
                    }
                    "log" => {
                        eb.add(Log(String::new()));
                    }
                    a => {
                        println!("Unused behavior: {a}")
                    }
                }
            }
        }
        if !self.dynamic_comps.is_empty() {
            println!("Unused components:");
            for i in &self.dynamic_comps {
                println!("\t {:?}", i);
            }
        }
        eb
    }
}

impl Resources {
    pub fn new() -> Self {
        Self {
            assets: Assets::new(),
            entity_templates: EntityTemplates::new(),
            body_templates: BodyTemplates::new(),
            item_templates: ItemTemplates::new(),
        }
    }
    pub async fn load(&mut self, mod_path: &Path) -> &mut Self {
        let templates_path = mod_path.join("templates");
        let gfx_path = mod_path.join("gfx");
        self.assets.load(&gfx_path).await;
        self.entity_templates.load(&templates_path);
        self.body_templates.load(&templates_path);
        self.item_templates.load(&templates_path);
        self
    }

    pub fn template_entity(&self, template_name: &str) -> EntityBuilder {
        self.entity_templates
            .template(&self.body_templates, template_name)
    }
    pub fn template_body(&self, template_name: &str) -> Body {
        self.body_templates.template(template_name)
    }
    pub fn template_item(&self, template_name: &str) -> Item {
        let template = self
            .item_templates
            .templates
            .get(template_name)
            .expect("this item template isnt exist");
        Item {
            name: template_name.into(),
            sprite_name: template.sprite_name.clone(),
            properties: template.properties.clone(),
            flags: template.flags.clone(),
        }
    }
    pub fn template_mob(&self) {}
}
