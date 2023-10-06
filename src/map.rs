use std::{collections::BTreeMap, sync::Arc, time::UNIX_EPOCH};

use random::Source;

#[derive(Clone, Debug)]
pub struct Tile {
    pub name: String,
    pub full_sprite: String,
    pub partial_sprite: Option<String>,
}

impl Tile {
    pub fn new(name: &str, sprite_name: &str) -> Self {
        Tile {
            name: name.into(),
            full_sprite: sprite_name.into(),
            partial_sprite: None,
        }
    }
    pub fn with_partial(&self, sprite_name: &str) -> Self {
        Tile {
            name: self.name.clone(),
            full_sprite: self.full_sprite.clone(),
            partial_sprite: Some(sprite_name.into()),
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

pub trait Map {
    type Chunk;
    fn xy_chunk(x: i32, y: i32) -> (i32, i32) {
        (
            (x.abs() + 7) * x.signum() / 15,
            (y.abs() + 7) * y.signum() / 15,
        )
    }
    fn xy_index_chunk(x: i32, y: i32) -> usize {
        let ch_x = x.rem_euclid(15);
        let ch_y = y.rem_euclid(15);
        let index = (ch_x + 7) % 15 + (ch_y + 7) % 15 * 15;
        index as usize
    }

    fn get_chunk_or_create(&mut self, x: i32, y: i32) -> &Self::Chunk;
    fn get_chunk(&self, x: i32, y: i32) -> Option<&Self::Chunk>;
    fn get_chunk_or_create_mut(&mut self, x: i32, y: i32) -> &mut Self::Chunk;
    fn get_chunk_mut(&mut self, x: i32, y: i32) -> Option<&mut Self::Chunk>;
}

pub struct WorldMap {
    chunks: BTreeMap<(i32, i32), Chunk>,
}

impl WorldMap {
    pub fn new() -> Self {
        WorldMap {
            chunks: BTreeMap::new(),
        }
    }
    pub fn get_obstacle_or_create(&mut self, x: i32, y: i32) -> bool {
        let (ch_x, ch_y) = Self::xy_chunk(x, y);
        let chunk = self.get_chunk_or_create(ch_x, ch_y);
        let idx = Self::xy_index_chunk(x, y);
        chunk.obstacles[idx]
    }
}

impl Map for WorldMap {
    fn get_chunk_or_create(&mut self, x: i32, y: i32) -> &Chunk {
        self.chunks.entry((x, y)).or_insert_with(Chunk::new)
    }
    fn get_chunk(&self, x: i32, y: i32) -> Option<&Chunk> {
        self.chunks.get(&(x, y))
    }
    fn get_chunk_or_create_mut(&mut self, x: i32, y: i32) -> &mut Chunk {
        self.chunks.entry((x, y)).or_insert_with(Chunk::new)
    }
    fn get_chunk_mut(&mut self, x: i32, y: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(x, y))
    }

    type Chunk = Chunk;
}

impl Chunk {
    pub fn new() -> Self {
        let height = 15;
        let width = 15;
        let mut tiles = Vec::new();
        let mut obstacles = Vec::new();
        let mut rnd = random::default(
            (std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros()
                % u64::max_value() as u128) as u64,
        );
        let wall_tile =
            Arc::new(Tile::new("brick wall", "brick_wall").with_partial("brick_wall_part"));
        let floor_tile = Arc::new(Tile::new("floor", "floor"));
        for _ in 0..height {
            for _ in 0..width {
                if rnd.read::<u32>() % 3000 == 0 {
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
