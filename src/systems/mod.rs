use hecs::World;
use tetra::Context;

use crate::Game;

pub mod error;
pub mod fov_compute;
pub mod memory;
pub mod move_player;
pub mod render;

#[macro_export]
macro_rules! init_systems {
    [$($system:expr),*] => {
        vec![$(Box::new($system)),*]
    };
}

pub type Result = std::result::Result<(), self::error::Error>;

/// Типаж систем, которые никак не влияют на игровой мир, и ориентированы на взаимодействие с игроком.
/// Примером служит система рендеринга.
pub trait GameSystem {
    fn run(&self, game: &Game, ctx: &mut Context) -> self::Result;
}

/// Типаж систем, которые определяют взаимодействия сущностей во внутреигровом мире, а
/// также обрабатывают действия игрока
pub trait WorldSystem {
    fn run(&self, world: &World, ctx: &Context) -> self::Result;
}
