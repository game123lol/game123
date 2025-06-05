use hecs::{CommandBuffer, Entity, With, World};
use vek::Vec3;

use crate::{components::Position, map::WorldMap, need_components, Direction, Mob};

// #[derive(Debug)]
// pub struct WantsMove(pub Direction);

pub const fn dir_to_vec3(dir: &Direction) -> Vec3<i32> {
    match dir {
        Direction::Forward => Vec3::new(0, -1, 0),
        Direction::Back => Vec3::new(0, 1, 0),
        Direction::Left => Vec3::new(-1, 0, 0),
        Direction::Right => Vec3::new(1, 0, 0),
        Direction::Up => Vec3::new(0, 0, 1),
        Direction::Down => Vec3::new(0, 0, -1),
    }
}

pub fn vec3_to_dir(vec: &Vec3<i32>) -> Option<Direction> {
    match vec.into_tuple() {
        (0, -1, 0) => Some(Direction::Forward),
        (0, 1, 0) => Some(Direction::Back),
        (-1, 0, 0) => Some(Direction::Left),
        (1, 0, 0) => Some(Direction::Right),
        (0, 0, 1) => Some(Direction::Up),
        (0, 0, -1) => Some(Direction::Down),
        _ => None,
    }
}

pub fn run_move(entity: Entity, dir: &Direction, world: &mut World) -> anyhow::Result<()> {
    let mut binding = world.query::<&Position>().with::<&Mob>();
    let mobs_pos = binding.iter().map(|a| a.1 .0).collect::<Vec<_>>();
    drop(binding);
    let mut binding = world.query_one::<&mut Position>(entity)?;
    let Position(pos) = binding.get().unwrap();
    let mut binding = world.query::<&mut WorldMap>();
    let (_, map) = binding
        .into_iter()
        .next()
        .ok_or(need_components!(MoveSystem, Map))?;
    let step = dir_to_vec3(dir) + *pos;
    if mobs_pos.contains(&step) {
        return Ok(());
    }
    if !map.get_obstacle_or_create(step.x, step.y, step.z) {
        *pos = step;
    }
    drop(binding);
    Ok(())
}
