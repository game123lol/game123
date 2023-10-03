use std::collections::HashSet;

use tetra::math::Vec2;

use crate::components::{MapMemory, Mob, Player, Position, Renderable, Sight};

pub fn new_player() -> (Position, Sight, Renderable, Player, Mob, MapMemory) {
    (
        Position(Vec2::new(1, 1)),
        Sight(30, HashSet::new()),
        Renderable("person".into()),
        Player,
        Mob,
        MapMemory::new(),
    )
}
