use {
    random::Source,
    std::{
        collections::BTreeMap,
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
    pub tiles: [Arc<Tile>; 225],
    pub obstacles: [bool; 225],
}

const fn const_xy_chunk(x: i32, y: i32) -> (i32, i32) {
    (
        ((x % 15) / 8 + x / 15),
        ((y % 15) / 8 + y / 15),
        //(x.abs() + 7) * x.signum() / 15,
        //(y.abs() + 7) * y.signum() / 15,
    )
}

const fn const_xy_index_chunk(x: i32, y: i32) -> usize {
    let ch_x = x.rem_euclid(15);
    let ch_y = y.rem_euclid(15);
    let index = (ch_x + 7) % 15 + (ch_y + 7) % 15 * 15;
    index as usize
}

pub trait Map {
    type Chunk;
    /// Функция, которая вычисляет координаты чанка от глобальной координаты
    #[inline]
    fn xy_chunk(x: i32, y: i32) -> (i32, i32) {
        const_xy_chunk(x, y)
    }
    #[inline]
    fn xy_index_chunk(x: i32, y: i32) -> usize {
        const_xy_index_chunk(x, y)
    }

    fn get_chunk_or_create(&mut self, x: i32, y: i32) -> &Mutex<Self::Chunk>;
    fn get_chunk(&self, x: i32, y: i32) -> Option<&Mutex<Self::Chunk>>;
}

pub struct WorldMap {
    chunks: BTreeMap<(i32, i32), Mutex<Chunk>>,
}

impl WorldMap {
    pub const fn new() -> Self {
        WorldMap {
            chunks: BTreeMap::new(),
        }
    }
    pub fn get_obstacle_or_create(&mut self, x: i32, y: i32) -> bool {
        let (ch_x, ch_y) = Self::xy_chunk(x, y);
        let chunk = self.get_chunk_or_create(ch_x, ch_y).lock().unwrap();
        let idx = Self::xy_index_chunk(x, y);

        chunk.obstacles[idx]
    }
}

impl Map for WorldMap {
    fn get_chunk_or_create(&mut self, x: i32, y: i32) -> &Mutex<Chunk> {
        self.chunks
            .entry((x, y))
            .or_insert_with(|| Mutex::new(Chunk::new()))
    }
    fn get_chunk(&self, x: i32, y: i32) -> Option<&Mutex<Chunk>> {
        self.chunks.get(&(x, y))
    }
    type Chunk = Chunk;
}

impl Chunk {
    pub fn new() -> Self {
        let height = 15;
        let width = 15;
        let mut tiles = Vec::with_capacity(225);
        let mut obstacles = Vec::with_capacity(225);
        let mut rnd = random::default(
            (std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros()
                % u64::max_value() as u128) as u64,
        );
        let wall_tile = Arc::new(Tile::new("cobblestone", "cobblestone").with_fallback("grass"));
        let floor_tile = Arc::new(Tile::new("grass", "grass"));
        for _ in 0..height {
            for _ in 0..width {
                if rnd.read::<u32>() % 300 == 0 {
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
