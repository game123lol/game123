use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tetra::{
    graphics::{Rectangle, Texture},
    Context,
};

pub struct ResourcesConfig {
    pub textures: Vec<TextureConfig>,
}

pub struct TextureConfig {
    pub source_file: PathBuf,
    pub sprite_size: (u32, u32),
    pub sprites: Vec<SpriteConfig>,
}

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
    pub fn new(config: &ResourcesConfig, ctx: &mut Context) -> Self {
        let mut textures = Vec::new();
        let mut sprites = HashMap::new();
        for texture_config in config.textures.iter() {
            let texture = Arc::new(Texture::new(ctx, texture_config.source_file.clone()).unwrap());
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
