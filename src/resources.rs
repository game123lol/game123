use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    num::ParseIntError,
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
use serde_yaml::Value;
use vek::Vec3;

use crate::{
    components::Position,
    mob::{Inventory, Log},
    systems::{fov_compute::Sight, memory::MapMemory, pathfinding::Pathfinder, render::Renderable},
    Mob,
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

pub struct Assets {
    pub sprites: HashMap<Arc<str>, Sprite>,
    pub textures: Vec<Rc<Texture2D>>,
}

pub struct Resources {
    pub assets: Assets,
    pub entity_templates: BTreeMap<Arc<str>, EntityBuilder>,
}

impl Resources {
    pub async fn load(data_path: &Path) -> Self {
        let entity_templates = Self::load_templates(data_path);
        Self {
            assets: Assets::load(&data_path.join("gfx")).await,
            entity_templates,
        }
    }
    pub fn load_templates(data_path: &Path) -> BTreeMap<Arc<str>, EntityBuilder> {
        let mut entity_templates = BTreeMap::new();
        let file = fs::read_to_string(data_path.join("templates.yaml")).unwrap();
        let templates: BTreeMap<String, Vec<Value>> = serde_yaml::from_str(&file).unwrap();
        for (template_name, template) in templates {
            let mut eb = EntityBuilder::new();
            for component in template {
                match component {
                    Value::String(ref compo_name) => match compo_name.as_str() {
                        "mob" => {
                            eb.add(Mob);
                        }
                        "log" => {
                            eb.add(Log("".into()));
                        }
                        "pathfinder" => {
                            eb.add(Pathfinder);
                        }
                        "inventory" => {
                            eb.add(Inventory(Vec::new()));
                        }
                        "map_memory" => {
                            eb.add(MapMemory::new());
                        }
                        _ => {
                            dbg!(component);
                            panic!("Уберите это немедленно")
                        }
                    },
                    Value::Mapping(ref mapping) => {
                        if mapping.len() != 1 {
                            dbg!(component);
                            panic!("Уберите это немедленно");
                        }
                        if let Some((Value::String(name), val)) = mapping.iter().next() {
                            match &(name.as_str(), val) {
                                // ("health", Value::Number(n)) => {
                                //     eb.add(DummyHealth(n.as_i64().unwrap() as i32));
                                // }
                                ("sight", Value::Number(n)) => {
                                    eb.add(Sight(n.as_u64().unwrap() as u32, HashSet::new()));
                                }
                                ("position", Value::String(pos_str)) => {
                                    let nums = pos_str
                                        .split('x')
                                        .map(|x| x.parse::<i32>())
                                        .collect::<Result<Vec<i32>, ParseIntError>>()
                                        .expect("Координаты должны быть в таком формате: XxYxZ");
                                    if nums.len() != 3 {
                                        panic!("Координата позиции трёхмерная должна быть");
                                    }
                                    eb.add(Position(Vec3::new(nums[0], nums[1], nums[2])));
                                }
                                ("renderable", Value::String(str)) => {
                                    eb.add(Renderable(str.to_owned().into()));
                                }
                                _ => {
                                    dbg!(component);
                                    panic!("Уберите это немедленно");
                                }
                            }
                        }
                    }
                    _ => {
                        dbg!(component);
                        panic!("Уберите это немедленно")
                    }
                }
            }

            entity_templates.insert(template_name.to_owned().into(), eb);
        }
        entity_templates
    }
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
        let mut textures = Vec::new();
        let mut sprites = HashMap::new();
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
            textures.push(texture.clone());
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
        Assets { sprites, textures }
    }
}
