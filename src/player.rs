use std::collections::HashSet;

use tetra::math::Vec2;

use crate::components::{Mob, Player, Position, Renderable, Sight};

pub fn new_player() -> (Position, Sight, Renderable, Player, Mob) {
    (
        Position(Vec2::new(1, 1)),
        Sight(HashSet::new()),
        Renderable("person".into()),
        Player,
        Mob,
    )
}
