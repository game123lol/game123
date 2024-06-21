use std::collections::HashMap;

use hecs::{Entity, World};
use rand::{seq::IteratorRandom, Rng};

use crate::mob::Log;

/// Компонент, временно выполняющий роль здоровья у мобов
/// Позже планируется заменить его на полноценную систему конечностей и органов
pub struct Health(pub i32);

/// Компонент, который появляется у сущности, атакующей в данной момент какую-то другую сущность
/// Предполагается, что он будет появляться от действий игрока или ИИ
#[derive(Copy, Clone)]
pub struct WantsAttack(pub Wound, pub Entity);

pub struct Body {
    parts: HashMap<String, BodyPart>,
}

impl Body {
    pub fn new() -> Self {
        Self {
            parts: HashMap::new(),
        }
    }
    pub fn with_part(mut self, part_name: String, part: BodyPart) -> Self {
        self.parts.insert(part_name, part);
        self
    }
    pub fn add_part(&mut self, part_name: String, part: BodyPart) {
        self.parts.insert(part_name, part);
    }
}

pub struct BodyPart {
    parts: HashMap<String, BodyPartPart>,
}

impl BodyPart {
    pub fn new() -> Self {
        Self {
            parts: HashMap::new(),
        }
    }
    pub fn with_part(mut self, part_name: String, part: BodyPartPart) -> Self {
        self.parts.insert(part_name, part);
        self
    }
    pub fn add_part(&mut self, part_name: String, part: BodyPartPart) {
        self.parts.insert(part_name, part);
    }
}

pub struct BodyPartPart {
    bone_groups: HashMap<String, BoneGroup>,
    skin: SkinPart,
    muscles: MuscleGroup,
    organs: HashMap<String, Organ>,
}

impl BodyPartPart {
    pub fn new() -> Self {
        Self {
            bone_groups: HashMap::new(),
            skin: SkinPart::new(),
            muscles: MuscleGroup::new(),
            organs: HashMap::new(),
        }
    }
    pub fn with_organ(mut self, organ_name: String, organ: Organ) -> Self {
        self.organs.insert(organ_name, organ);
        self
    }
    pub fn add_organ(&mut self, organ_name: String, organ: Organ) {
        self.organs.insert(organ_name, organ);
    }
    pub fn with_bone_group(mut self, bone_group_name: String, bone_group: BoneGroup) -> Self {
        self.bone_groups.insert(bone_group_name, bone_group);
        self
    }
    pub fn add_bone_group(&mut self, bone_group_name: String, bone_group: BoneGroup) {
        self.bone_groups.insert(bone_group_name, bone_group);
    }
}

pub struct MuscleGroup {
    wounds: Vec<Wound>,
}

impl SkinPart {
    pub fn new() -> Self {
        Self { wounds: Vec::new() }
    }
}

impl MuscleGroup {
    pub fn new() -> Self {
        Self { wounds: Vec::new() }
    }
}

pub struct SkinPart {
    wounds: Vec<Wound>,
}

pub struct Organ {
    wounds: Vec<Wound>,
}

impl Organ {
    pub fn new() -> Self {
        Self { wounds: Vec::new() }
    }
}

#[derive(Copy, Clone)]
pub enum Wound {
    Incised,
    Stabbed,
    Lacerated,
    Bitten,
    Bruised,
    Gunshot,
    Scalped,
    Surgical,
}

pub struct BoneGroup {
    fractures: Vec<Fracture>,
}

pub enum Fracture {
    Open,
    Closed,
}

pub fn test() {}

pub fn run_attack_system(world: &mut World) -> anyhow::Result<()> {
    let mut attackers_bind = world.query::<(&WantsAttack,)>();
    let attackers: Vec<_> = attackers_bind.iter().map(|(e, (a,))| (e, *a)).collect();
    drop(attackers_bind);
    for (e, WantsAttack(damage, target)) in attackers.iter() {
        let mut log = String::new();
        if let Ok((target_body,)) = world.query_one_mut::<(&mut Body,)>(*target) {
            let mut rng = rand::thread_rng();
            let target_part = target_body.parts.iter_mut().choose(&mut rng).unwrap();
            //TODO: рандомизировать урон
            //TODO: убрать полный рандом, сделать возможность прицеливаться для удара
            let target_part_part = target_part.1.parts.iter_mut().choose(&mut rng).unwrap();

            let organs_count = target_part_part.1.organs.len();
            let target_organs_count = rng.gen_range(0..organs_count / 3);
            let mut target_organs = target_part_part
                .1
                .organs
                .iter_mut()
                .choose_multiple(&mut rng, target_organs_count);
            let target_bone_group = target_part_part
                .1
                .bone_groups
                .iter_mut()
                .choose(&mut rng)
                .unwrap();
            log.push_str("You are bruising something, you have received wounds: ");
            for organ in target_organs.iter_mut() {
                organ.1.wounds.push(*damage);
                log.push_str(format!("{} ", organ.0).as_str());
            }
            target_part_part.1.muscles.wounds.push(*damage);
            target_part_part.1.skin.wounds.push(*damage);
            // FIXME добавить более продвинутую обработку ран
            target_bone_group.1.fractures.push(Fracture::Closed);
            log.push_str(format!("and {} fracture", target_bone_group.0).as_str());
        }
        if let Ok((attacker_log,)) = world.query_one_mut::<(&mut Log,)>(*e) {
            attacker_log.0.push_str(log.as_str());
        }
        world.remove_one::<WantsAttack>(*e)?;
    }
    Ok(())
}
