use std::collections::{BTreeMap, BTreeSet};

use tetra::math::Vec2;

use crate::map::Map;

#[derive(Debug)]
pub struct Renderable(pub String);

pub struct Player;

pub struct Mob;

pub struct Position(pub Vec2<i32>);

pub struct Item;

pub struct Sight(pub u32, pub BTreeSet<(i32, i32)>);

pub struct Name(pub String);

pub struct MapMemory {
    chunks: BTreeMap<(i32, i32), MemoryChunk>,
}

impl MapMemory {
    pub fn new() -> Self {
        MapMemory {
            chunks: BTreeMap::new(),
        }
    }
}

pub struct MemoryChunk {
    pub memorized: [bool; 255],
}

impl MemoryChunk {
    pub fn new() -> Self {
        MemoryChunk {
            memorized: [false; 255],
        }
    }
}

impl Map for MapMemory {
    fn get_chunk_or_create(&mut self, x: i32, y: i32) -> &MemoryChunk {
        self.chunks.entry((x, y)).or_insert_with(MemoryChunk::new)
    }
    fn get_chunk(&self, x: i32, y: i32) -> Option<&MemoryChunk> {
        self.chunks.get(&(x, y))
    }
    fn get_chunk_or_create_mut(&mut self, x: i32, y: i32) -> &mut MemoryChunk {
        self.chunks.entry((x, y)).or_insert_with(MemoryChunk::new)
    }
    fn get_chunk_mut(&mut self, x: i32, y: i32) -> Option<&mut MemoryChunk> {
        self.chunks.get_mut(&(x, y))
    }

    type Chunk = MemoryChunk;
}
