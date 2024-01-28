use std::sync::Mutex;

use crate::{
    components::{MapMemory, MemoryChunk, Player, Position, Sight},
    map::Map,
    need_components,
};

pub fn run_memory_system(world: &hecs::World, _ctx: &tetra::Context) -> super::Result {
    let mut query = world.query::<(&Player, &Position, &Sight, &mut MapMemory)>();
    let (_, (_, Position(cam_pos), Sight(_, sight_tiles), map_memory)) =
        query.iter().next().ok_or(need_components!(
            MemorySystem,
            Player,
            Position,
            Sight,
            MapMemory
        ))?;
    let shift_back = |pos: (i32, i32)| (pos.0 + cam_pos.x, pos.1 + cam_pos.y);

    let mut chunk_cache: Vec<((i32, i32), *const Mutex<MemoryChunk>)> = Vec::new();
    for sight_coord in sight_tiles.iter() {
        let (x, y) = shift_back(*sight_coord);
        let (ch_x, ch_y) = MapMemory::xy_chunk(x, y);
        let chunk_mutex = map_memory.get_chunk_or_create(ch_x, ch_y);
        //        let chunk_mutex =
        //            if let Some((_, chunk)) = chunk_cache.iter().rev().find(|a| a.0 == (ch_x, ch_y)) {
        //                unsafe { chunk.as_ref().unwrap() }
        //            } else {
        //                let link = map_memory.get_chunk_or_create(ch_x, ch_y);
        //                chunk_cache.push(((ch_x, ch_y), link));
        //                link
        //            };
        let mut chunk = chunk_mutex.lock().unwrap();

        let real_crd = MapMemory::xy_index_chunk(x, y);
        chunk.memorized[real_crd] = true;
    }
    Ok(())
}
