use hecs::World;
use tetra::Context;

use crate::Game;

use self::{
    fov_compute::run_fov_compute_system, input::run_input_system, memory::run_memory_system,
    move_player::run_move_system,
};

pub mod error;
pub mod fov_compute;
pub mod input;
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

#[derive(Clone, Copy)]
pub enum GameSystem {
    InputSystem,
}
#[derive(Clone, Copy)]
pub enum WorldSystem {
    FovCompute,
    Move,
    Memory,
}

impl WorldSystem {
    pub fn run(&self, world: &mut World, ctx: &mut Context) -> anyhow::Result<()> {
        match self {
            WorldSystem::FovCompute => run_fov_compute_system(world, ctx)?,
            WorldSystem::Move => run_move_system(world)?,
            WorldSystem::Memory => run_memory_system(world, ctx)?,
        }
        Ok(())
    }
}

impl GameSystem {
    pub fn run(&self, game: &mut Game, ctx: &mut Context) -> std::result::Result<(), error::Error> {
        match self {
            GameSystem::InputSystem => run_input_system(game, ctx),
        }
    }
}
