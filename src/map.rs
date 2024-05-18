use std::collections::HashMap;

use vek::Vec3;

use {
    random::Source,
    std::{
        sync::{Arc, Mutex},
        time::UNIX_EPOCH,
    },
};

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
    pub const fn with_fallback(&self, sprite_name: &'static str) -> Self {
        Tile {
            name: self.name,
            full_sprite: self.full_sprite,
            fallback_sprite: Some(sprite_name),
        }
    }
}

// Карта - это объект, в котором хранится какое-то количество загруженных чанков
// У каждого чанка есть свои декартовы координаты, и чанк собой являет линейный массив тайлов
// У тайла есть декартовы координаты, но тайл можно получить только по индексу.

#[derive(Clone)]
pub struct Chunk {
    pub tiles: [Arc<Tile>; 3375], //15x15x15
    pub obstacles: [bool; 3375],
}

const fn const_xy_chunk(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    (
        ((x % 15) / 8 + x / 15),
        ((y % 15) / 8 + y / 15),
        ((z % 15) / 8 + z / 15),
    )
}

const fn const_xy_index_chunk(x: i32, y: i32, z: i32) -> usize {
    let ch_x = x.rem_euclid(15);
    let ch_y = y.rem_euclid(15);
    let ch_z = z.rem_euclid(15);
    let index = (ch_x + 7) % 15 + (ch_y + 7) % 15 * 15 + (ch_z + 7) % 15 * 225;
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
        let size = Vec3::new(15, 15, 15);
        let mut tiles = Vec::with_capacity(225);
        let mut obstacles = Vec::with_capacity(225);
        let mut rnd = random::default(
            (std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros()
                % u64::max_value() as u128) as u64,
        );

        let is_sphere_in_chunk = rnd.read::<u32>() % 5 == 0;
        let in_sphere = {
            let radius = rnd.read::<u16>() as i32 % 2 + 3;
            let x_crd = rnd.read::<u16>() as i32 % (15 - radius) + radius + 1;
            let y_crd = rnd.read::<u16>() as i32 % (15 - radius) + radius + 1;
            let z_crd = rnd.read::<u16>() as i32 % 3 + 6;
            move |x: i32, y: i32, z: i32| {
                (((x - x_crd) * (x - x_crd) + (y - y_crd) * (y - y_crd) + (z - z_crd) * (z - z_crd))
                    as f64)
                    .sqrt()
                    < radius as f64
            }
        };

        let wall_tile = Arc::new(Tile::new("wall", "wall"));
        let empty_tile = Arc::new(Tile::new("empty", "empty"));
        for z in 0..size.z {
            for y in 0..size.y {
                for x in 0..size.x {
                    let is_wall = ch_z < 0
                        || (ch_z == 0 && (z < 7 || (z == 7 && rnd.read::<u32>() % 300 == 0)))
                        || (ch_z == 0 && is_sphere_in_chunk && in_sphere(x, y, z));
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
