use std::collections::BTreeSet;

use tetra::math::Vec2;

use crate::components::{MapMemory, Mob, Player, Position, Renderable, Sight};

pub const fn new_player() -> (Position, Sight, Renderable, Player, Mob, MapMemory) {
    (
        Position(Vec2::new(1, 1)),
        Sight(50, BTreeSet::new()),
        Renderable("person"),
        Player,
        Mob,
        MapMemory::new(),
    )
}
