use hecs::{CommandBuffer, World};
use pathfinding::prelude::*;
use serde::Deserialize;
use vek::Vec3;

use crate::{
    components::Position,
    map::{Map, WorldMap},
    need_components,
    player::Player,
    Direction, Mob,
};

use super::movement::{dir_to_vec3, vec3_to_dir, WantsMove};

// type Path = (Vec<Vec3<i32>>, i32);

pub struct Pathfinder;

const fn mhdistance(a: &Vec3<i32>, b: &Vec3<i32>) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()
}

pub fn run_pathfinding_system(world: &mut World) -> anyhow::Result<()> {
    let mut cmd = CommandBuffer::new();
    {
        // let mut mobs_bind = world.query::<(&Mob, &Position)>();
        // let mobs = mobs_bind.iter();
        let mut movables = world.query::<(&Position, &Mob, &Pathfinder)>();
        let mut binding = world.query::<(&mut WorldMap,)>();
        let (_, (map,)) = binding
            .iter()
            .next()
            .ok_or(need_components!(Pathfinding, Map))?;
        let mut binding = world.query::<(&Player, &Position)>();
        let (_, (_, Position(player_pos))) = binding
            .iter()
            .next()
            .ok_or(need_components!(Pathfinding, Player))?;

        let dirs = [
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
            Direction::Forward,
            Direction::Back,
        ];

        let sucsessors = |pos: &Vec3<i32>| {
            let mut result = Vec::new();
            for (x, y, z) in dirs.iter().map(dir_to_vec3).map(|x| x.into_tuple()) {
                let (pos_x, pos_y, pos_z) = (pos.x + x, pos.y + y, pos.z + z);
                let (ch_x, ch_y, ch_z) = WorldMap::xy_chunk(pos_x, pos_y, pos_z);
                let chunk_mutex = map.get_chunk(ch_x, ch_y, ch_z).unwrap();

                let chunk = chunk_mutex.lock().unwrap();
                let real = WorldMap::xy_index_chunk(pos_x, pos_y, pos_z);
                let is_obstacle = chunk.obstacles[real];
                if !is_obstacle {
                    let res_pos = Vec3::new(pos_x, pos_y, pos_z);
                    let distance = 1;
                    result.push((res_pos, distance));
                }
            }
            result
        };

        let distance = |pos: &Vec3<i32>| mhdistance(player_pos, pos);

        for (e, (Position(pos), _, _)) in movables.iter() {
            let a = astar(pos, sucsessors, distance, |x| x == player_pos);
            if let Some((path, _)) = a {
                let next_step = path[1] - pos;
                if let Some(dir) = vec3_to_dir(&next_step) {
                    cmd.insert_one(e, WantsMove(dir));
                }
            }
        }
    }
    cmd.run_on(world);

    Ok(())
}
