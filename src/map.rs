use random::Source;
use tetra::{graphics::Rectangle, math::Vec2};

#[derive(Clone)]
pub struct Sprite {
    pub src_name: String,
    pub rect: Rectangle,
}

pub struct Tile {
    pub name: String,
    pub full_sprite: Sprite,
    pub partial_sprite: Option<Sprite>,
}

impl Tile {
    pub fn new(name: &str, texture_name: &str, x: u8, y: u8) -> Self {
        Tile {
            name: name.into(),
            full_sprite: Sprite {
                src_name: texture_name.into(),
                rect: Rectangle::new(x as f32 * 16., y as f32 * 20., 16., 20.),
            },
            partial_sprite: None,
        }
    }
    pub fn with_partial(&self, texture_name: &str, x: u8, y: u8) -> Self {
        Tile {
            name: self.name.clone(),
            full_sprite: self.full_sprite.clone(),
            partial_sprite: Some(Sprite {
                src_name: texture_name.into(),
                rect: Rectangle::new(x as f32 * 16., y as f32 * 20., 16., 20.),
            }),
        }
    }
}

pub struct Map {
    pub size: (usize, usize),
    pub tiles: Vec<Tile>,
    pub obstacles: Vec<bool>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = Vec::new();
        let mut obstacles = Vec::new();
        let mut rnd = random::default(42);
        for i in 0..height {
            for j in 0..width {
                if i == 0
                    || j == 0
                    || i == height - 1
                    || j == width - 1
                    || rnd.read::<u32>() % 10 == 0
                {
                    let tile =
                        Tile::new("wall", "tileset_iso", 0, 0).with_partial("tileset_iso", 1, 0);
                    tiles.push(tile);
                    obstacles.push(true);
                } else {
                    let tile = Tile::new("floor", "tileset_iso", 0, 1);
                    tiles.push(tile);
                    obstacles.push(false)
                }
            }
        }
        Map {
            size: (width, height),
            tiles,
            obstacles,
        }
    }
    pub fn xy_index(&self, x: i32, y: i32) -> usize {
        (x + y * self.size.0 as i32) as usize
    }
    pub fn xy_index_safe(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }
        let x = x as usize;
        let y = y as usize;
        if x > self.size.0 || y > self.size.1 {
            return None;
        }
        Some(x + y * self.size.0)
    }
}
