use hecs::{CommandBuffer, Entity, World};

use crate::{components::Position, items::Item, mob::Log};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Item with index {0} not exist")]
    ItemNotExist(usize),
}

#[derive(Clone, Debug)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub holded: Option<usize>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            holded: None,
        }
    }
    pub fn equip(&mut self, index: usize) -> Result<(), Error> {
        if index >= self.items.len() {
            return Err(Error::ItemNotExist(index));
        }
        self.holded = Some(index);
        Ok(())
    }
    pub fn dequip(&mut self) {
        self.holded = None;
    }
    pub fn get_holded_item(&self) -> Option<&Item> {
        self.holded.map(|index| self.items.get(index)).flatten()
    }
}

pub fn run_pickup_item(
    entity: Entity,
    item_entity: Entity,
    world: &mut World,
) -> Result<(), Error> {
    let item = {
        let mut binding = world.query_one::<&Item>(item_entity).unwrap();
        binding.get().unwrap().clone()
    };
    let (inventory, log) = world
        .query_one_mut::<(&mut Inventory, Option<&mut Log>)>(entity)
        .unwrap();
    dbg!(&log);
    if let Some(log) = log {
        log.write(format!("Подобран предмет {}", &item.name).as_str());
    }
    inventory.items.push(item);
    world.despawn(item_entity).unwrap();

    Ok(())
}

pub fn run_equip_item(entity: Entity, index: usize, world: &mut World) -> Result<(), Error> {
    let (inventory, log) = world
        .query_one_mut::<(&mut Inventory, Option<&mut Log>)>(entity)
        .unwrap();
    if let Some(log) = log {
        let item = &inventory.items[index];
        log.write(format!("Экипирован предмет {}", item.name).as_str());
    }
    inventory.equip(index)?;
    Ok(())
}

pub fn run_dequip_item(entity: Entity, world: &World) {
    let mut binding = world.query_one::<&mut Inventory>(entity).unwrap();
    let inventory = binding.get().unwrap();
    inventory.dequip();
}

pub fn run_drop_item(entity: Entity, index: usize, world: &mut World) -> Result<(), Error> {
    dbg!("there");
    let (inventory, log, pos) = world
        .query_one_mut::<(&mut Inventory, Option<&mut Log>, Option<&Position>)>(entity)
        .unwrap();
    if index >= inventory.items.len() {
        return Err(Error::ItemNotExist(index));
    }
    dbg!(&log);
    if let Some(log) = log {
        let item = &inventory.items[index];
        log.write(format!("Выброшен предмет {}", item.name).as_str());
    }
    let item = inventory.items.remove(index);
    if let Some(holded) = &mut inventory.holded {
        if index < *holded {
            *holded = holded.saturating_sub(1);
        }
        if index == 0 || index == *holded {
            inventory.holded = None;
        }
    }
    let mut cmd = CommandBuffer::new();
    if let Some(pos) = pos {
        cmd.spawn(item.into_map_entity(&pos));
    }
    cmd.run_on(world);
    Ok(())
}
