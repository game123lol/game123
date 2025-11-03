use hecs::{Entity, World};

use crate::{
    body::{run_attack, Wound},
    inventory::{run_dequip_item, run_drop_item, run_equip_item, run_pickup_item},
    Direction,
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

impl Action {
    pub fn new(world: &World, duration: i64, action_kind: ActionKind) -> Self {
        let mut bind = world.query::<&WorldTime>();
        let (_, current_time) = bind.iter().last().unwrap();
        Self {
            end_time: current_time.0 + duration,
            action: action_kind,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ActionKind {
    Attack(Wound, Entity),
    Move(Direction),
    PickUp(Entity),
    Equip(usize),
    Dequip,
    DropItem(usize),
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
        ActionKind::PickUp(item) => run_pickup_item(*e, item, world).unwrap(),
        ActionKind::Equip(index) => run_equip_item(*e, index, world).unwrap(),
        ActionKind::Dequip => run_dequip_item(*e, world),
        ActionKind::DropItem(index) => run_drop_item(*e, index, world).unwrap(),
    }
    world.remove_one::<Action>(*e).unwrap();
    let mut binding = world.query::<&mut WorldTime>();
    let (_, time) = binding.iter().last().unwrap();
    time.0 = action.end_time;

    Ok(())
}
