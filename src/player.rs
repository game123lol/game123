use tetra::{graphics::Rectangle, math::Vec2};

use crate::entities::{Mob, Player, Position, Renderable};

pub fn new_player() -> (Position, Renderable, Player, Mob) {
    (
        Position(Vec2::new(1, 1)),
        Renderable("person".into(), Rectangle::new(0., 0., 16., 16.)),
        Player,
        Mob,
    )
}