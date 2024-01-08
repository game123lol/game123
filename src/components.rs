use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

use crate::{map::Map, Direction};
use hecs::Entity;
use tetra::math::Vec2;

/// Компонент, используемый в функции рендера. Все сущности, обладающие этим компонентом,
/// а так же компонентами Position и Item или Mob, будут отрисованы.
/// Компонент содержит в себе название спрайта, который будет отрисован.
/// По этому названию будет сделан запрос в хранилище спрайтов resources (поле Game).
#[derive(Debug)]
pub struct Renderable(pub &'static str);

/// Компонент, означающий, что сущность с этим компонентом - управляема игроком.
/// Ожидается, что она должна встречаться только один раз в игре.
pub struct Player;

pub struct Inventory(pub Vec<Entity>);

/// Компонент, означающий, что сущность с этим компонентом - как-либо действующиее
/// существо. Это может быть игрок или неигровой персонаж.
pub struct Mob;

/// Компонент, который должен быть у сущностей, которые будут иметь позицию на
/// игровой карте. Это может быть, например, лежащий на земле предмет, игрок или неигровой персонаж.
pub struct Position(pub Vec2<i32>);

pub struct WantsMove(pub Direction);

/// Компонент, маркер предмета. Сущность, которая обладает этим компонентом, должна иметь позицию,
/// если она находится на карте, или же она должна находиться в чьём-нибудь инвентаре.
pub struct Item;

/// Компонент, означающий, что сущность с этим компонентом имеет поле зрения.
/// Он имеет в себе радиус поля зрения и множество координат, которые сущность видит.
pub struct Sight(pub u32, pub BTreeSet<(i32, i32)>);

/// Компонент, имя какой-либо сущности.
pub struct Name(pub Arc<str>);

/// Компонент, означающий, что сущность запоминает тайлы, которые увидела однажды
/// Хранит в себе карту, где вместо соответствующих тайлов содержатся булевы значения.
pub struct MapMemory {
    chunks: BTreeMap<(i32, i32), Mutex<MemoryChunk>>,
}

impl MapMemory {
    pub const fn new() -> Self {
        MapMemory {
            chunks: BTreeMap::new(),
        }
    }
}

pub struct MemoryChunk {
    pub memorized: [bool; 255],
}

impl MemoryChunk {
    pub const fn new() -> Self {
        MemoryChunk {
            memorized: [false; 255],
        }
    }
}

impl Map for MapMemory {
    fn get_chunk_or_create(&mut self, x: i32, y: i32) -> &Mutex<MemoryChunk> {
        self.chunks
            .entry((x, y))
            .or_insert_with(|| Mutex::new(MemoryChunk::new()))
    }
    fn get_chunk(&self, x: i32, y: i32) -> Option<&Mutex<MemoryChunk>> {
        self.chunks.get(&(x, y))
    }

    type Chunk = MemoryChunk;
}
