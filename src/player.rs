use std::collections::HashSet;

use tetra::{graphics::Rectangle, math::Vec2};

use crate::entities::{Mob, Player, Position, Renderable, Sight};

pub fn new_player() -> (Position, Sight, Renderable, Player, Mob) {
    (
        Position(Vec2::new(1, 1)),
        Sight(HashSet::new()),
        Renderable("person".into(), Rectangle::new(0., 0., 16., 16.)),
        Player,
        Mob,
    )
}
