use hecs::World;

use crate::Game;

use self::{
    fov_compute::run_fov_compute_system, health::run_attack_system, input::run_input_system,
    memory::run_memory_system, movement::run_move_system, pathfinding::run_pathfinding_system,
};

pub mod error;
pub mod fov_compute;
pub mod health;
pub mod input;
pub mod memory;
pub mod movement;
pub mod pathfinding;
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

#[derive(Clone, Copy, Debug)]
pub enum WorldSystem {
    FovCompute,
    Move,
    Memory,
    Attack,
    Pathfinding,
}

impl WorldSystem {
    pub fn run(&self, world: &mut World) -> anyhow::Result<()> {
        match self {
            WorldSystem::FovCompute => run_fov_compute_system(world)?,
            WorldSystem::Move => run_move_system(world)?,
            WorldSystem::Memory => run_memory_system(world)?,
            WorldSystem::Attack => run_attack_system(world)?,
            WorldSystem::Pathfinding => run_pathfinding_system(world)?,
        }
        Ok(())
    }
}

impl GameSystem {
    pub fn run(&self, game: &mut Game) -> anyhow::Result<()> {
        match self {
            GameSystem::InputSystem => run_input_system(game)?,
        }
        Ok(())
    }
}
