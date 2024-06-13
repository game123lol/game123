use std::{collections::HashMap, sync::Mutex};

use crate::{
    components::Position,
    map::{self, Chunk, Map, WorldMap, CHUNK_SIZE},
    need_components,
    player::Player,
};

use super::fov_compute::Sight;

/// Компонент, означающий, что сущность запоминает тайлы, которые увидела однажды
/// Хранит в себе карту, где вместо соответствующих тайлов содержатся булевы значения.
pub struct MapMemory {
    chunks: HashMap<(i32, i32, i32), Mutex<MemoryChunk>>,
}

impl MapMemory {
    pub fn new() -> Self {
        MapMemory {
            chunks: HashMap::new(),
        }
    }
}

pub struct MemoryChunk {
    pub memorized: [bool; CHUNK_SIZE.pow(3)],
}

impl MemoryChunk {
    pub const fn new() -> Self {
        MemoryChunk {
            memorized: [false; CHUNK_SIZE.pow(3)],
        }
    }
    pub fn is_memorized(&self, x: i32, y: i32, z: i32) -> bool {
        let idx = MapMemory::xy_index_chunk(x, y, z);
        unsafe { *self.memorized.get_unchecked(idx) }
    }
}

impl Map for MapMemory {
    fn get_chunk_or_create(&mut self, x: i32, y: i32, z: i32) -> &Mutex<MemoryChunk> {
        self.chunks
            .entry((x, y, z))
            .or_insert_with(|| Mutex::new(MemoryChunk::new()))
    }
    fn get_chunk(&self, x: i32, y: i32, z: i32) -> Option<&Mutex<MemoryChunk>> {
        self.chunks.get(&(x, y, z))
    }

    type Chunk = MemoryChunk;
}

pub fn run_memory_system(world: &hecs::World) -> super::Result {
    let mut query = world.query::<(&mut WorldMap,)>();
    let (_, (map,)) = query
        .iter()
        .next()
        .ok_or(need_components!(FovSystem, WorldMap))?;

    let mut query = world.query::<(&Player, &Position, &Sight, &mut MapMemory)>();
    let (_, (_, Position(cam_pos), Sight(_, sight_tiles), map_memory)) =
        query.iter().next().ok_or(need_components!(
            MemorySystem,
            Player,
            Position,
            Sight,
            MapMemory
        ))?;
    let shift_back =
        |pos: (i32, i32, i32)| (pos.0 + cam_pos.x, pos.1 + cam_pos.y, pos.2 + cam_pos.z);

    for ((x, y, z), _) in map.chunks.iter() {
        map_memory.get_chunk_or_create(*x, *y, *z);
    }

    let mut chunk_cache: [Option<(&Mutex<MemoryChunk>, i32, i32, i32)>; 15] = [None; 15];
    let mut cache_counter = 0;
    for sight_coord in sight_tiles.iter() {
        let (x, y, z) = shift_back(*sight_coord);
        let (ch_x, ch_y, ch_z) = MapMemory::xy_chunk(x, y, z);
        let chunk_mutex = match chunk_cache
            .into_iter()
            .find(|x| x.is_some_and(|(_, x, y, z)| x == ch_x && y == ch_y && ch_z == z))
        {
            Some(Some((mutex, _, _, _))) => mutex,
            _ => {
                let new_chunk_mutex = map_memory.get_chunk(ch_x, ch_y, ch_z).unwrap();
                chunk_cache[cache_counter] = Some((new_chunk_mutex, ch_x, ch_y, ch_z));
                cache_counter += 1;
                cache_counter %= 15;
                new_chunk_mutex
            }
        };
        let mut chunk = chunk_mutex.lock().unwrap();

        let real_crd = MapMemory::xy_index_chunk(x, y, z);
        chunk.memorized[real_crd] = true;
    }
    Ok(())
}
