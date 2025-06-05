use hecs::{CommandBuffer, Entity, World};

use crate::{
    body::{run_attack, Wound},
    format_entity, Direction,
};

use super::movement::run_move;

/// Компонент, отвечающий за внутриигровое время, содержащий количество секунд,
/// которое прошло от создания мира
#[derive(Clone)]
pub struct WorldTime(pub i64);

/// Компонент, означающий, что сущность в данный момент времени совершает какое-то действие
#[derive(Clone, Debug)]
pub struct Action {
    pub end_time: i64,
    pub action: ActionKind,
}

#[derive(Clone, Debug)]
pub enum ActionKind {
    Attack(Wound, Entity),
    Move(Direction),
}

pub fn run_action_system(world: &mut World) -> super::Result {
    let actions = {
        let mut binding = world.query::<&Action>();
        binding
            .iter()
            .map(|a| (a.0, a.1.clone()))
            .collect::<Vec<_>>()
    };
    let Some((e, action)) = actions.iter().min_by_key(|a| a.1.end_time) else {
        return Ok(());
    };
    match action.action {
        ActionKind::Attack(wound, target) => {
            run_attack(*e, target, wound, world).unwrap();
        }
        ActionKind::Move(dir) => {
            run_move(*e, &dir, world).unwrap();
        }
    }
    world.remove_one::<Action>(*e).unwrap();
    let mut binding = world.query::<&mut WorldTime>();
    let (_, time) = binding.iter().last().unwrap();
    time.0 = action.end_time;

    Ok(())
}
