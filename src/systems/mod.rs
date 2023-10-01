use hecs::World;
use tetra::Context;

use crate::Game;

pub mod error;
pub mod fov_compute;
pub mod move_player;
pub mod render;

#[macro_export]
macro_rules! init_systems {
    [$($system:expr),*] => {
        vec![$(Box::new($system)),*]
    };
}

pub type Result = std::result::Result<(), self::error::Error>;

pub trait GameSystem {
    fn run(&self, game: &Game, ctx: &mut Context) -> self::Result;
}

pub trait WorldSystem {
    fn run(&self, world: &World, ctx: &Context) -> self::Result;
}
