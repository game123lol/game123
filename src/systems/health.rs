use hecs::{Entity, World};

use crate::player::Log;

/// Компонент, временно выполняющий роль здоровья у мобов
/// Позже планируется заменить его на полноценную систему конечностей и органов
pub struct DummyHealth(pub i32);

/// Компонент, который появляется у сущности, атакующей в данной момент какую-то другую сущность
/// Предполагается, что он будет появляться от действий игрока или ИИ
#[derive(Copy, Clone)]
pub struct WantsAttack(pub Damage, pub Entity);

// Позже, числовое значение будет заменено на более продвинутый тип урона
#[derive(Copy, Clone)]
pub struct Damage(pub i32);

pub fn run_attack_system(world: &mut World) -> anyhow::Result<()> {
    let mut attackers_bind = world.query::<(&WantsAttack,)>();
    let attackers: Vec<_> = attackers_bind.iter().map(|(e, (a,))| (e, *a)).collect();
    drop(attackers_bind);
    for (e, WantsAttack(damage, target)) in attackers.iter() {
        let mut damaged_health = None;
        if let Ok((target_health,)) = world.query_one_mut::<(&mut DummyHealth,)>(*target) {
            target_health.0 -= damage.0;
            damaged_health = Some(target_health.0);
        }
        if let Ok((log,)) = world.query_one_mut::<(&mut Log,)>(*e) {
            if let Some(damaged_health) = damaged_health {
                log.write(
                    format!(
                        "Attacked something, now his dummyhealth is {}",
                        damaged_health
                    )
                    .as_str(),
                );
            }
        }
        world.remove_one::<WantsAttack>(*e)?;
    }
    Ok(())
}
