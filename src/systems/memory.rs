use crate::{
    components::{MapMemory, Player, Position, Sight},
    map::Map,
    need_components,
};

use super::WorldSystem;

pub struct MemorySystem;

impl WorldSystem for MemorySystem {
    fn run(&self, world: &hecs::World, _ctx: &tetra::Context) -> super::Result {
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
        for sight_coord in sight_tiles.iter() {
            let (x, y) = shift_back(*sight_coord);
            let (ch_x, ch_y) = MapMemory::xy_chunk(x, y);
            let chunk = map_memory.get_chunk_or_create_mut(ch_x, ch_y);
            let real_crd = MapMemory::xy_index_chunk(x, y);
            chunk.memorized[real_crd] = true;
        }
        Ok(())
    }
}
