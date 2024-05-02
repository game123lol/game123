use std::{collections::BTreeMap, sync::Mutex};

use crate::{components::Position, map::Map, need_components, player::Player};

use super::fov_compute::Sight;

/// Компонент, означающий, что сущность запоминает тайлы, которые увидела однажды
/// Хранит в себе карту, где вместо соответствующих тайлов содержатся булевы значения.
pub struct MapMemory {
    chunks: BTreeMap<(i32, i32, i32), Mutex<MemoryChunk>>,
}

impl MapMemory {
    pub const fn new() -> Self {
        MapMemory {
            chunks: BTreeMap::new(),
        }
    }
}

pub struct MemoryChunk {
    pub memorized: [bool; 3375],
}

impl MemoryChunk {
    pub const fn new() -> Self {
        MemoryChunk {
            memorized: [false; 3375],
        }
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

    //let mut chunk_cache: Vec<((i32, i32), *const Mutex<MemoryChunk>)> = Vec::new();
    for sight_coord in sight_tiles.iter() {
        let (x, y, z) = shift_back(*sight_coord);
        let (ch_x, ch_y, ch_z) = MapMemory::xy_chunk(x, y, z);
        let chunk_mutex = map_memory.get_chunk_or_create(ch_x, ch_y, ch_z);
        //        let chunk_mutex =
        //            if let Some((_, chunk)) = chunk_cache.iter().rev().find(|a| a.0 == (ch_x, ch_y)) {
        //                unsafe { chunk.as_ref().unwrap() }
        //            } else {
        //                let link = map_memory.get_chunk_or_create(ch_x, ch_y);
        //                chunk_cache.push(((ch_x, ch_y), link));
        //                link
        //            };
        let mut chunk = chunk_mutex.lock().unwrap();

        let real_crd = MapMemory::xy_index_chunk(x, y, z);
        chunk.memorized[real_crd] = true;
    }
    Ok(())
}
