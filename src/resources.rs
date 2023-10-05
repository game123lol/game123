use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use tetra::{
    graphics::{Rectangle, Texture},
    Context,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResourcesConfig {
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
    pub rect: Rectangle,
    pub texture: Arc<Texture>,
}

pub struct Resources {
    pub sprites: HashMap<String, Sprite>,
    pub textures: Vec<Arc<Texture>>,
}

impl Resources {
    pub fn load(ctx: &mut Context, assets_path: &Path) -> Self {
        let config_path = assets_path.join("assets.json");
        let json_config = fs::read_to_string(config_path).unwrap_or_else(|_| {
            panic!(
                "File assets.json not found in {} directory",
                assets_path.display()
            )
        });
        let config: ResourcesConfig =
            serde_json::from_str(&json_config).expect("assets.json file is corrupted");
        Self::new(ctx, &config, assets_path)
    }
    pub fn new(ctx: &mut Context, config: &ResourcesConfig, assets_path: &Path) -> Self {
        let mut textures = Vec::new();
        let mut sprites = HashMap::new();
        for texture_config in config.textures.iter() {
            let texture = Arc::new(
                Texture::new(ctx, assets_path.join(texture_config.source_file.clone())).unwrap(),
            );
            textures.push(texture.clone());
            for sprite_config in texture_config.sprites.iter() {
                let sprite = Sprite {
                    rect: Rectangle::new(
                        sprite_config.coords.0 as f32 * texture_config.sprite_size.0 as f32,
                        sprite_config.coords.1 as f32 * texture_config.sprite_size.1 as f32,
                        texture_config.sprite_size.0 as f32,
                        texture_config.sprite_size.1 as f32,
                    ),
                    texture: texture.clone(),
                };
                sprites.insert(sprite_config.name.clone(), sprite);
            }
        }
        Resources { sprites, textures }
    }
}
