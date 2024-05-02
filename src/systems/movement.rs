use hecs::{CommandBuffer, World};
use tetra::math::{Vec2, Vec3};

use crate::{components::Position, map::WorldMap, need_components, Direction, Mob};

pub struct WantsMove(pub Direction);

pub const fn dir_to_vec2(dir: &Direction) -> Vec3<i32> {
    match dir {
        Direction::Forward => Vec3::new(0, -1, 0),
        Direction::Back => Vec3::new(0, 1, 0),
        Direction::Left => Vec3::new(-1, 0, 0),
        Direction::Right => Vec3::new(1, 0, 0),
        Direction::Up => Vec3::new(0, 0, 1),
        Direction::Down => Vec3::new(0, 0, -1),
    }
}

pub fn run_move_system(world: &mut World) -> anyhow::Result<()> {
    let mut mobs_bind = world.query::<(&Mob, &Position)>();
    let mobs: Vec<_> = mobs_bind
        .iter()
        .map(|(e, (_, Position(pos)))| (e.id(), *pos))
        .collect();
    drop(mobs_bind);
    let mut movables = world.query::<(&mut Position, &WantsMove)>();
    let mut binding = world.query::<(&mut WorldMap,)>();
    let (_, (map,)) = binding
        .into_iter()
        .next()
        .ok_or(need_components!(MovePlayerSystem, Map))?;
    let mut cmd = CommandBuffer::new();
    let mut next_steps = movables
        .iter()
        .map(|(e, (Position(pos), WantsMove(dir)))| (e.id(), *pos + dir_to_vec2(dir)))
        .collect::<Vec<_>>();
    next_steps.dedup_by(|a, b| a.1 == b.1);
    for (e, (Position(pos), _)) in movables.iter() {
        cmd.remove_one::<WantsMove>(e);
        // Если в потенциально занятых позициях есть сущность e
        if let Some((_, step)) = next_steps.iter().find(|a| a.0 == e.id()) {
            // И позиция, куда она хочет идти, занята мобом
            if let Some((collision_mob_id, _)) = mobs.iter().find(|a| a.1 == *step) {
                // Который никуда не двигается
                if next_steps.iter().any(|a| a.0 != *collision_mob_id) {
                    // То не двигать её
                    continue;
                }
            }
            // Иначе если на пути нет препятствия
            if !map.get_obstacle_or_create(step.x, step.y, step.z) {
                // То двигать
                *pos = *step;
            }
        }
    }
    drop(movables);
    drop(binding);
    cmd.run_on(world);
    Ok(())
}
