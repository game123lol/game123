use hecs::{CommandBuffer, World};
use tetra::math::Vec2;

use crate::{components::Position, map::WorldMap, need_components, Direction};

pub struct WantsMove(pub Direction);

pub fn run_move_system(world: &mut World) -> anyhow::Result<()> {
    let mut movables = world.query::<(&mut Position, &WantsMove)>();
    let mut binding = world.query::<(&mut WorldMap,)>();
    let (_, (map,)) = binding
        .into_iter()
        .next()
        .ok_or(need_components!(MovePlayerSystem, Map))?;
    let mut cmd = CommandBuffer::new();
    for (e, (Position(pos), WantsMove(dir))) in movables.into_iter() {
        let mut step = *pos;
        match dir {
            Direction::Up => {
                step += Vec2::new(0, -1);
            }
            Direction::Down => {
                step += Vec2::new(0, 1);
            }
            Direction::Left => {
                step += Vec2::new(-1, 0);
            }
            Direction::Right => {
                step += Vec2::new(1, 0);
            }
        }
        if !map.get_obstacle_or_create(step.x, step.y) {
            *pos = step;
        }
        cmd.remove::<(&WantsMove,)>(e); //FIXME: Компонент не удаляется
        dbg!("removed wantsmove from", e);
    }
    drop(movables);
    drop(binding);
    cmd.run_on(world);
    Ok(())
}
