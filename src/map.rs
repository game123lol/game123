use std::{collections::HashMap, sync::Arc};

use random::Source;
use tetra::graphics::Rectangle;

#[derive(Clone, Debug)]
pub struct Sprite {
    pub src_name: String,
    pub rect: Rectangle,
}

#[derive(Clone, Debug)]
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

// Карта - это объект, в котором хранится какое-то количество загруженных чанков
// У каждого чанка есть свои декартовы координаты, и чанк собой являет линейный массив тайлов
// У тайла есть декартовы координаты, но тайл можно получить только по индексу.

pub struct Chunk {
    pub tiles: [Arc<Tile>; 225],
    pub obstacles: [bool; 225],
}

pub struct Map {
    chunks: HashMap<(i32, i32), Chunk>,
}

impl Map {
    pub fn new() -> Self {
        Map {
            chunks: HashMap::new(),
        }
    }
    pub fn xy_chunk(x: i32, y: i32) -> (i32, i32) {
        let x_sig = x.signum();
        let y_sig = y.signum();
        let x_abs = if x.abs() < 7 {
            0
        } else {
            (x.abs() - 7) / 15 + 1
        };
        let y_abs = if y.abs() < 7 {
            0
        } else {
            (y.abs() - 7) / 15 + 1
        };
        (x_abs * x_sig, y_abs * y_sig)
    }
    pub fn get_chunk_or_create(&mut self, x: i32, y: i32) -> &Chunk {
        if !self.chunks.contains_key(&(x, y)) {
            let chunk = Chunk::new();
            self.chunks.insert((x, y), chunk);
            println!("chunk {} {} created", x, y);
        }
        self.chunks.get(&(x, y)).unwrap()
    }
    pub fn get_chunk(&self, x: i32, y: i32) -> &Chunk {
        self.chunks.get(&(x, y)).unwrap()
    }
    pub fn xy_index_chunk(x: i32, y: i32) -> usize {
        let mut ch_x = x % 15;
        let mut ch_y = y % 15;
        if ch_x <= -8 {
            ch_x += 15;
        }
        if ch_x >= 8 {
            ch_x -= 15;
        }
        if ch_y <= -8 {
            ch_y += 15;
        }
        if ch_y >= 8 {
            ch_y -= 15;
        }
        let index = (ch_x + 7) + (ch_y + 7) * 15;
        index as usize
    }
    pub fn get_obstacle_or_create(&mut self, x: i32, y: i32) -> bool {
        let (ch_x, ch_y) = Self::xy_chunk(x, y);
        let chunk = self.get_chunk_or_create(ch_x, ch_y);
        let idx = Self::xy_index_chunk(x, y);
        chunk.obstacles[idx]
    }
}

impl Chunk {
    pub fn new() -> Self {
        let height = 15;
        let width = 15;
        let mut tiles = Vec::new();
        let mut obstacles = Vec::new();
        let mut rnd = random::default(42);
        let wall_tile =
            Arc::new(Tile::new("wall", "tileset_iso", 0, 0).with_partial("tileset_iso", 1, 0));
        let floor_tile = Arc::new(Tile::new("floor", "tileset_iso", 0, 1));
        for _ in 0..height {
            for _ in 0..width {
                if rnd.read::<u32>() % 30 == 0 {
                    tiles.push(wall_tile.clone());
                    obstacles.push(true);
                } else {
                    tiles.push(floor_tile.clone());
                    obstacles.push(false)
                }
            }
        }
        Chunk {
            tiles: tiles.try_into().unwrap(),
            obstacles: obstacles.try_into().unwrap(),
        }
    }
}
