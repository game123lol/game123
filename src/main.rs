mod components;
mod items;
mod map;
mod player;
mod resources;
mod systems;
mod tests;
use components::Position;
use egui_tetra::{
    egui::{self, Pos2},
    StateWrapper,
};
use hecs::{CommandBuffer, World};
use items::{Item, Property};
use map::WorldMap;
use player::{get_player_items, new_player, Inventory, Log, Player};
use resources::Resources;
use std::{collections::HashMap, env, sync::Arc};
use systems::{
    health::{Damage, DummyHealth},
    movement::{dir_to_vec3, WantsMove},
    pathfinding::Pathfinder,
    render::{run_render_system, Renderable},
    GameSystem, WorldSystem,
};
use tetra::{
    graphics::{self, scaling::ScreenScaler, Color},
    input::Key,
    window, Context, ContextBuilder,
};

use crate::systems::health::WantsAttack;

type GameSystems = Vec<GameSystem>;
type WorldSystems = Vec<WorldSystem>;

type Type = World;

enum UIState {
    No,
    Inventory { items: Vec<Item> },
    Log { text: String },
    Debug,
}

/// Компонент, означающий, что сущность с этим компонентом - как-либо действующиее
/// существо. Это может быть игрок или неигровой персонаж.
pub struct Mob;

pub type DialogKeys = HashMap<Key, PlayerAction>;

pub struct UIConfig {
    dialogs_keys: HashMap<String, DialogKeys>,
    world_keys: HashMap<Key, PlayerAction>,
}

impl UIConfig {
    fn default() -> Self {
        let mut dialogs_keys = HashMap::new();
        let mut inventory_keys = HashMap::new();
        inventory_keys.insert(Key::Q, PlayerAction::CloseInventory);
        dialogs_keys.insert("inventory".into(), inventory_keys);
        let mut log_keys = HashMap::new();
        log_keys.insert(Key::Q, PlayerAction::CloseLog);
        dialogs_keys.insert("log".into(), log_keys);

        let mut world_keys = HashMap::new();

        world_keys.insert(Key::H, PlayerAction::Move(Direction::Left));
        world_keys.insert(Key::J, PlayerAction::Move(Direction::Back));
        world_keys.insert(Key::K, PlayerAction::Move(Direction::Forward));
        world_keys.insert(Key::L, PlayerAction::Move(Direction::Right));
        world_keys.insert(Key::U, PlayerAction::Move(Direction::Up));
        world_keys.insert(Key::N, PlayerAction::Move(Direction::Down));
        world_keys.insert(Key::I, PlayerAction::OpenInventory);
        world_keys.insert(Key::E, PlayerAction::PickUpItem);
        world_keys.insert(Key::P, PlayerAction::OpenLog);

        Self {
            dialogs_keys,
            world_keys,
        }
    }
}

pub struct Game {
    world: Type,
    resources: Resources,
    scaler: ScreenScaler,
    game_systems: GameSystems,
    world_systems: WorldSystems,
    ui_state: UIState,
    ui_config: UIConfig,
    next_action: PlayerAction,
    is_paused: bool,
    is_needed_redraw: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerAction {
    Move(Direction),
    OpenInventory,
    CloseInventory,
    OpenLog,
    CloseLog,
    PickUpItem,
    Nothing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Back,
    Left,
    Right,
    Up,
    Down,
}

enum Action {
    PlayerAction(PlayerAction),
    UIEvent {},
}

impl egui_tetra::State<anyhow::Error> for Game {
    fn ui(&mut self, ctx: &mut tetra::Context, egui_ctx: &egui::CtxRef) -> anyhow::Result<()> {
        match &self.ui_state {
            UIState::Inventory { items } => {
                egui::Window::new("Inventory")
                    .fixed_pos(Pos2::new(1., 1.))
                    .show(egui_ctx, |ui| {
                        ui.label("Items in inventory:");
                        for item in items {
                            ui.label(item.name.clone());
                        }
                    });
            }
            UIState::No => {}
            UIState::Debug => {
                let fps = tetra::time::get_fps(ctx);
                egui::Window::new("Debug")
                    .fixed_pos(Pos2::new(1., 1.))
                    .show(egui_ctx, |ui| {
                        ui.label(format!("fps: {}", fps));
                    });
            }
            UIState::Log { text } => {
                egui::Window::new("Log")
                    .fixed_pos(Pos2::new(1., 1.))
                    .show(egui_ctx, |ui| {
                        for event in text.split('\n') {
                            ui.label(event);
                        }
                    });
            }
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context, _egui_ctx: &egui::CtxRef) -> anyhow::Result<()> {
        let (w, h) = window::get_size(ctx);
        self.scaler.set_outer_size(w, h);
        if self.is_needed_redraw && self.is_paused {
            let now = std::time::Instant::now();

            graphics::set_canvas(ctx, self.scaler.canvas());
            graphics::clear(ctx, Color::rgb(0., 0., 0.));
            run_render_system(self, ctx).unwrap();
            graphics::reset_canvas(ctx);
            graphics::clear(ctx, Color::rgb(0., 0., 0.));
            self.is_needed_redraw = false;
            let elapsed = now.elapsed();
            println!("Render elapsed: {:.2?}", elapsed);
        }
        self.scaler.draw(ctx);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context, _egui_ctx: &egui::CtxRef) -> anyhow::Result<()> {
        if self.is_paused {
            match self.next_action {
                PlayerAction::Move(dir) => {
                    let mut bind = self.world.query::<(&Player, &Position)>();
                    let (e, (_, Position(pos))) = bind
                        .into_iter()
                        .next()
                        .expect("Персонаж потерялся. Как так?");
                    let pos = *pos;
                    drop(bind);
                    let mut mobs = self.world.query::<(&Mob, &Position)>();
                    let mut cmd = CommandBuffer::new();
                    if let Some((target, _)) = mobs
                        .iter()
                        .find(|(_, (_, Position(mob_pos)))| *mob_pos == pos + dir_to_vec3(&dir))
                    {
                        cmd.insert(e, (WantsAttack(Damage(1), target),));
                    } else {
                        cmd.insert(e, (WantsMove(dir),));
                    }
                    drop(mobs);
                    cmd.run_on(&mut self.world);
                    self.is_paused = false;
                    self.is_needed_redraw = true;
                }
                PlayerAction::OpenInventory => {
                    self.ui_state = UIState::Inventory {
                        items: get_player_items(&self.world)?,
                    }
                }
                PlayerAction::PickUpItem => {
                    let mut bind_player = self
                        .world
                        .query::<(&Player, &Position, &mut Inventory, &mut Log)>();
                    let (_, (_, player_pos, inventory, log)) = bind_player
                        .into_iter()
                        .next()
                        .expect("Персонаж потерялся. Как так?");
                    let mut bind_item = self.world.query::<(&Item, &Position)>();
                    let items = bind_item.into_iter();
                    let mut cmd = CommandBuffer::new();
                    for (e, (item, pos)) in items {
                        if *pos == *player_pos {
                            log.write(&format!("Picked up {}", item.name.clone()));
                            inventory.0.push(item.clone());
                            cmd.despawn(e);
                            break;
                        }
                    }
                    drop(bind_item);
                    drop(bind_player);
                    cmd.run_on(&mut self.world);
                    self.is_paused = false;
                    self.is_needed_redraw = true;
                }
                PlayerAction::OpenLog => {
                    let mut bind_player = self.world.query::<(&Player, &Log)>();
                    let (_, (_, log)) = bind_player
                        .into_iter()
                        .next()
                        .expect("Персонаж потерялся. Как так?");
                    self.ui_state = UIState::Log {
                        text: log.0.clone(),
                    }
                }
                PlayerAction::CloseLog | PlayerAction::CloseInventory => {
                    self.ui_state = UIState::No;
                }
                _ => {}
            }
            for system in self.game_systems.clone().iter() {
                system.run(self, ctx)?
            }
        } else {
            let now = std::time::Instant::now();
            for system in self.world_systems.iter() {
                system.run(&mut self.world)?
            }
            let elapsed = now.elapsed();
            println!("World systems elapsed: {:.2?}", elapsed);

            self.is_paused = true;
        }
        Ok(())
    }
}

impl Game {
    fn new(ctx: &mut Context) -> tetra::Result<Game> {
        let exe_path = env::current_exe().expect("Ты ебанутый? Ты что там делаешь?");
        let assets_path = exe_path
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("assets"))
            .filter(|p| p.exists())
            .unwrap_or(
                env::current_dir()
                    .expect("Ты как сюда залез?")
                    .join("assets"),
            );
        let game_systems: GameSystems = vec![GameSystem::InputSystem];
        let world_systems: WorldSystems = vec![
            WorldSystem::Move,
            WorldSystem::FovCompute,
            WorldSystem::Memory,
            WorldSystem::Pathfinding,
            WorldSystem::Attack,
        ];
        let resources = Resources::load(ctx, &assets_path);
        let mut world = World::new();
        let map = WorldMap::new();
        world.spawn((map,));
        world.spawn(new_player());
        let mut item = Item::new("thing".into(), "item".into());
        item.add_props(&[("huy".into(), Property::Marker)]);
        world.spawn(item.to_map_entity(2, 2, 0));
        let nettle = (
            Position(tetra::math::Vec3::new(10, 10, 0)),
            Renderable(Arc::from("nettle")),
            Mob,
            DummyHealth(3),
            Pathfinder,
        );
        world.spawn(nettle);
        let scaler = ScreenScaler::new(
            ctx,
            1000,
            1000,
            1000,
            1000,
            graphics::scaling::ScalingMode::CropPixelPerfect,
        )
        .unwrap();
        Ok(Game {
            world,
            resources,
            scaler,
            game_systems,
            world_systems,
            ui_state: UIState::Debug,
            ui_config: UIConfig::default(),
            next_action: PlayerAction::Nothing,
            is_paused: false,
            is_needed_redraw: true,
        })
    }
}

pub fn main() -> anyhow::Result<()> {
    ContextBuilder::new("S", 500, 500)
        .quit_on_escape(true)
        .resizable(true)
        .show_mouse(true)
        .grab_mouse(false)
        .key_repeat(true)
        .build()?
        .run(|ctx| Ok(StateWrapper::new(Game::new(ctx)?)))?;
    Ok(())
}
