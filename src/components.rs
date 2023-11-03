use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

use tetra::math::Vec2;

use crate::map::Map;

#[derive(Debug)]
pub struct Renderable(pub &'static str);

pub struct Player;

pub struct Mob;

pub struct Position(pub Vec2<i32>);

pub struct Item;

pub struct Sight(pub u32, pub BTreeSet<(i32, i32)>);

pub struct Name(pub Arc<str>);

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
