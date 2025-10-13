// #TODO: Заменить на mods/core

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use hecs::EntityBuilder;
use macroquad::{
    prelude::Rect,
    texture::{load_texture, Texture2D},
};

use serde::{Deserialize, Serialize};
use vek::Vec3;

use crate::{
    body::{Body, BodyPart, BodyPartPart, BoneGroup, Organ},
    components::Position,
    hasher,
    inventory::Inventory,
    items::Item,
    mob::Log,
    systems::{fov_compute::Sight, memory::MapMemory, pathfinding::Pathfinder, render::Renderable},
    GameHasher, Mob, Property,
};

#[derive(Serialize, Deserialize)]
pub struct AssetsConfig {
    pub textures: Vec<TextureConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct TextureConfig {
    pub source_file: PathBuf,
    pub sprite_size: (u32, u32),
    pub sprites: Vec<SpriteConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct SpriteConfig {
    pub coords: (u8, u8),
    pub name: String,
}

#[derive(Debug)]
pub struct Sprite {
    pub rect: Rect,
    pub texture: Rc<Texture2D>,
}

#[derive(Debug)]
pub struct Assets {
    pub sprites: HashMap<Arc<str>, Sprite, GameHasher>,
}

pub struct Resources {
    pub assets: Assets,
    pub entity_templates: EntityTemplates,
    pub body_templates: BodyTemplates,
    pub item_templates: ItemTemplates,
}

#[derive(Debug, Deserialize)]
pub struct ItemTemplates {
    #[serde(flatten)]
    pub templates: HashMap<String, ItemTemplate, GameHasher>,
}

#[derive(Debug, Deserialize)]
pub struct ItemTemplate {
    #[serde(rename = "sprite")]
    pub sprite_name: String,
    #[serde(default)]
    pub properties: HashMap<String, Property, GameHasher>,
    #[serde(default)]
    pub flags: HashSet<String>,
}

impl ItemTemplates {
    pub fn load(data_path: &Path) -> Self {
        let file = fs::read_to_string(data_path.join("item_templates.yaml")).unwrap();
        serde_yaml::from_str(&file).unwrap()
    }
}

//TODO: либо убрать эту todo, либо переделать эту жесть
#[derive(Deserialize, Debug)]
struct ComponentTemplates {
    body: Option<String>,
    sprite: Option<String>,
    sight: Option<u32>,
    behaviors: Option<HashSet<String>>,
    // Только для тестов
    position: Option<(i32, i32, i32)>,
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
        if let Some(position) = &self.position {
            eb.add(Position(Vec3 {
                x: position.0,
                y: position.1,
                z: position.2,
            }));
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

#[derive(Debug, Deserialize)]
struct EntityTemplate {
    #[serde(flatten)]
    components: ComponentTemplates,
}

#[derive(Debug, Deserialize)]
pub struct EntityTemplates {
    #[serde(flatten)]
    templates: HashMap<String, EntityTemplate, GameHasher>,
}

//TODO убери анврапы, сделай норм ошибки через thiserror
impl EntityTemplates {
    pub fn load(data_path: &Path) -> Self {
        let file = fs::read_to_string(data_path.join("entity_templates.yaml")).unwrap();
        serde_yaml::from_str(&file).unwrap()
    }
    //TODO билдеры создаются каждый раз, их надо б как нибудь кэшировать чтоль
    pub fn template(&self, body_templates: &BodyTemplates, template_name: &str) -> EntityBuilder {
        self.templates
            .get(template_name)
            .unwrap()
            .components
            .to_entity_builder(body_templates)
    }
}

#[derive(Debug, Deserialize)]
pub struct BodyTemplates {
    #[serde(flatten)]
    pub templates: HashMap<String, BodyTemplate, GameHasher>,
}

#[derive(Debug, Deserialize)]
pub struct BodyTemplate {
    #[serde(flatten)]
    bodyparts: HashMap<String, BodyPartTemplate, GameHasher>,
}

impl BodyTemplate {
    fn new() -> Self {
        Self {
            bodyparts: HashMap::with_hasher(GameHasher::default()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BodyPartTemplate {
    #[serde(flatten)]
    bodypartparts: HashMap<String, BodyPartSegmentTemplate, GameHasher>,
}

impl BodyPartTemplate {
    fn new() -> Self {
        Self {
            bodypartparts: HashMap::with_hasher(GameHasher::default()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BodyPartSegmentTemplate {
    #[serde(default)]
    organs: Vec<String>,
    #[serde(default)]
    bone_groups: Vec<String>,
    #[serde(default)]
    properties: HashMap<String, Property, GameHasher>,
    #[serde(default)]
    flags: HashSet<String>,
}

impl BodyPartSegmentTemplate {
    fn new() -> Self {
        Self {
            organs: Vec::new(),
            bone_groups: Vec::new(),
            properties: HashMap::with_hasher(GameHasher::default()),
            flags: HashSet::new(),
        }
    }
}

impl BodyTemplates {
    pub fn load(data_path: &Path) -> Self {
        let file = fs::read_to_string(data_path.join("body_templates.yaml")).unwrap();
        serde_yaml::from_str(&file).unwrap()
    }
    fn template(&self, template_name: &str) -> Body {
        let template = self
            .templates
            .get(template_name)
            .expect("this body template isnt exist");
        let mut body = Body::new();
        for (name, part_template) in &template.bodyparts {
            let mut part = BodyPart::new();
            for (part_name, part_segment_template) in &part_template.bodypartparts {
                let mut part_segment = BodyPartPart::new();
                for i in part_segment_template.organs.iter() {
                    part_segment.add_organ(i.clone(), Organ::new())
                }
                for i in part_segment_template.bone_groups.iter() {
                    part_segment.add_bone_group(i.clone(), BoneGroup::new())
                }
                for i in part_segment_template.properties.iter() {
                    part_segment.add_property(i.0.clone(), i.1.to_owned())
                }
                part.add_part(part_name.clone(), part_segment);
            }
            body.add_part(name.clone(), part);
        }
        body
    }
}

impl Resources {
    pub async fn load(data_path: &Path) -> Self {
        let templates_path = data_path.join("templates");
        let gfx_path = data_path.join("gfx");
        let body_templates = BodyTemplates::load(&templates_path);
        let entity_templates = EntityTemplates::load(&templates_path);
        Self {
            assets: Assets::load(&gfx_path).await,
            entity_templates,
            body_templates,
            item_templates: ItemTemplates::load(&templates_path),
        }
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

impl Assets {
    pub async fn load(assets_path: &Path) -> Self {
        let config_path = assets_path.join("assets.yaml");
        let yaml_config = fs::read_to_string(config_path).unwrap_or_else(|_| {
            panic!(
                "File assets.yaml not found in {} directory",
                assets_path.display()
            )
        });
        let config: AssetsConfig =
            serde_yaml::from_str(&yaml_config).expect("assets.yaml file is corrupted");
        Self::new(&config, assets_path).await
    }
    pub async fn new(config: &AssetsConfig, assets_path: &Path) -> Self {
        let mut sprites = HashMap::with_hasher(hasher());
        for texture_config in config.textures.iter() {
            let texture = load_texture(
                assets_path
                    .join(texture_config.source_file.clone())
                    .as_os_str()
                    .to_str()
                    .unwrap(),
            )
            .await
            .unwrap();
            texture.set_filter(macroquad::texture::FilterMode::Nearest);
            let texture = Rc::new(texture);
            for sprite_config in texture_config.sprites.iter() {
                let sprite = Sprite {
                    rect: Rect::new(
                        sprite_config.coords.0 as f32 * texture_config.sprite_size.0 as f32,
                        sprite_config.coords.1 as f32 * texture_config.sprite_size.1 as f32,
                        texture_config.sprite_size.0 as f32,
                        texture_config.sprite_size.1 as f32,
                    ),
                    texture: texture.clone(),
                };
                sprites.insert(sprite_config.name.to_owned().into(), sprite);
            }
        }
        Assets { sprites }
    }
}
