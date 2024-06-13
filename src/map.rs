use std::collections::HashMap;

use rand::Rng;

use std::sync::{Arc, Mutex};

pub const CHUNK_SIZE: usize = 64;

/// Структура, содержащая информацию о тайле. Пока что она имеет только
/// такие параметры, как имя тайла и название спрайтов, которыми нужно
/// её отображать. Это будет изменено в ближайшее время.
#[derive(Clone, Debug)]
pub struct Tile {
    pub name: &'static str,
    /// имя спрайта, которым нужно отображать этот тайл
    pub full_sprite: &'static str,
    /// имя спрайта, который рисуется под full_sprite
    pub fallback_sprite: Option<&'static str>,
}

impl Tile {
    pub const fn new(name: &'static str, sprite_name: &'static str) -> Self {
        Tile {
            name,
            full_sprite: sprite_name,
            fallback_sprite: None,
        }
    }
}

// Карта - это объект, в котором хранится какое-то количество загруженных чанков
// У каждого чанка есть свои декартовы координаты, и чанк собой являет линейный массив тайлов
// У тайла есть декартовы координаты, но тайл можно получить только по индексу.

#[derive(Clone)]
pub struct Chunk {
    pub tiles: [Arc<Tile>; CHUNK_SIZE.pow(3)], //15x15x15
    pub obstacles: [bool; CHUNK_SIZE.pow(3)],
}

impl<'a> Chunk {
    pub fn get_tile(&'a self, x: i32, y: i32, z: i32) -> &'a Arc<Tile> {
        let idx = const_xy_index_chunk(x, y, z);
        unsafe { self.tiles.get_unchecked(idx) }
    }
    pub fn get_obstacle(&self, x: i32, y: i32, z: i32) -> bool {
        let idx = const_xy_index_chunk(x, y, z);
        unsafe { *self.obstacles.get_unchecked(idx) }
    }
}

const fn const_xy_chunk(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    (
        ((x % CHUNK_SIZE as i32) / (CHUNK_SIZE as i32 / 2) + x / CHUNK_SIZE as i32),
        ((y % CHUNK_SIZE as i32) / (CHUNK_SIZE as i32 / 2) + y / CHUNK_SIZE as i32),
        ((z % CHUNK_SIZE as i32) / (CHUNK_SIZE as i32 / 2) + z / CHUNK_SIZE as i32),
    )
}

const fn const_xy_index_chunk(x: i32, y: i32, z: i32) -> usize {
    let ch_x = x.rem_euclid(CHUNK_SIZE as i32);
    let ch_y = y.rem_euclid(CHUNK_SIZE as i32);
    let ch_z = z.rem_euclid(CHUNK_SIZE as i32);
    let index = (ch_x + CHUNK_SIZE as i32 / 2) % CHUNK_SIZE as i32
        + (ch_y + CHUNK_SIZE as i32 / 2) % CHUNK_SIZE as i32 * CHUNK_SIZE as i32
        + (ch_z + CHUNK_SIZE as i32 / 2) % CHUNK_SIZE as i32 * CHUNK_SIZE.pow(2) as i32;
    index as usize
}

pub trait Map {
    type Chunk;
    /// Функция, которая вычисляет координаты чанка от глобальной координаты
    #[inline]
    fn xy_chunk(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
        const_xy_chunk(x, y, z)
    }
    #[inline]
    fn xy_index_chunk(x: i32, y: i32, z: i32) -> usize {
        const_xy_index_chunk(x, y, z)
    }

    fn get_chunk_or_create(&mut self, x: i32, y: i32, z: i32) -> &Mutex<Self::Chunk>;
    fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<&Mutex<Self::Chunk>>;
}

pub struct WorldMap {
    pub chunks: HashMap<(i32, i32, i32), Mutex<Chunk>>,
}

impl WorldMap {
    pub fn new() -> Self {
        WorldMap {
            chunks: HashMap::new(),
        }
    }
    pub fn get_obstacle_or_create(&mut self, x: i32, y: i32, z: i32) -> bool {
        let (ch_x, ch_y, ch_z) = Self::xy_chunk(x, y, z);
        let chunk = self.get_chunk_or_create(ch_x, ch_y, ch_z).lock().unwrap();
        let idx = Self::xy_index_chunk(x, y, z);

        chunk.obstacles[idx]
    }
}

impl Map for WorldMap {
    fn get_chunk_or_create(&mut self, x: i32, y: i32, z: i32) -> &Mutex<Chunk> {
        self.chunks
            .entry((x, y, z))
            .or_insert_with(|| Mutex::new(Chunk::new(x, y, z)))
    }
    fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<&Mutex<Chunk>> {
        self.chunks.get(&(x, y, z))
    }
    type Chunk = Chunk;
}

impl Chunk {
    pub fn new(ch_x: i32, ch_y: i32, ch_z: i32) -> Self {
        let mut tiles = Vec::with_capacity(CHUNK_SIZE.pow(3));
        let mut obstacles = Vec::with_capacity(CHUNK_SIZE.pow(3));
        let mut rng = rand::thread_rng();

        let is_sphere_in_chunk = rng.gen_bool(1. / 5.);
        let in_sphere = {
            let radius = 2;

            let x_crd = rng.gen_range(radius..CHUNK_SIZE as i32 - radius);
            let y_crd = rng.gen_range(radius..CHUNK_SIZE as i32 - radius);
            let z_crd = rng.gen_range(-1..3);
            move |x: i32, y: i32, z: i32| {
                (((x - x_crd) * (x - x_crd) + (y - y_crd) * (y - y_crd) + (z - z_crd) * (z - z_crd))
                    as f64)
                    .sqrt()
                    < radius as f64
            }
        };

        let wall_tile = Arc::new(Tile::new("wall", "wall"));
        let empty_tile = Arc::new(Tile::new("empty", "empty"));
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let is_ground = ch_z < 0 || (z < CHUNK_SIZE / 2 && ch_z == 0);
                    let is_wall_on_ground = z == CHUNK_SIZE / 2 && rng.gen_bool(1. / 300.);
                    let is_sphere_on_ground =
                        is_sphere_in_chunk && ch_z == 0 && in_sphere(x as i32, y as i32, z as i32);
                    let is_wall = is_ground || is_wall_on_ground || is_sphere_on_ground;
                    if is_wall {
                        tiles.push(wall_tile.clone());
                        obstacles.push(true);
                    } else {
                        tiles.push(empty_tile.clone());
                        obstacles.push(false);
                    }
                }
            }
        }

        Chunk {
            tiles: tiles.try_into().unwrap(),
            obstacles: obstacles.try_into().unwrap(),
        }
    }
}
