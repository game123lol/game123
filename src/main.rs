mod components;
mod items;
mod map;
mod player;
mod resources;
mod systems;
mod tests;
use components::{Name, Position};
use egui_tetra::{
    egui::{self, Pos2},
    StateWrapper,
};
use hecs::{Entity, World};
use items::Item;
use map::WorldMap;
use player::{get_player_items, new_player, Player};
use resources::Resources;
use std::{collections::HashMap, env};
use systems::{
    movement::WantsMove,
    render::{run_render_system, Renderable},
    GameSystem, WorldSystem,
};
use tetra::{
    graphics::{self, scaling::ScreenScaler, Color},
    input::Key,
    math::Vec2,
    window, Context, ContextBuilder,
};

type GameSystems = Vec<GameSystem>;
type WorldSystems = Vec<WorldSystem>;

type Type = World;

enum UIState {
    No,
    Inventory { items: Vec<Entity> },
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

        let mut world_keys = HashMap::new();

        world_keys.insert(Key::H, PlayerAction::Move(Direction::Left));
        world_keys.insert(Key::J, PlayerAction::Move(Direction::Down));
        world_keys.insert(Key::K, PlayerAction::Move(Direction::Up));
        world_keys.insert(Key::L, PlayerAction::Move(Direction::Right));
        world_keys.insert(Key::I, PlayerAction::OpenInventory);

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    Move(Direction),
    OpenInventory,
    CloseInventory,
    Nothing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    //UpLeft,
    //UpRight,
    //DownLeft,
    //DownRight,
}

enum Action {
    PlayerAction(PlayerAction),
    UIEvent {},
}

impl egui_tetra::State<anyhow::Error> for Game {
    fn ui(&mut self, _ctx: &mut tetra::Context, egui_ctx: &egui::CtxRef) -> anyhow::Result<()> {
        match &self.ui_state {
            UIState::Inventory { items } => {
                egui::Window::new("Inventory")
                    .fixed_pos(Pos2::new(1., 1.))
                    .show(egui_ctx, |ui| {
                        ui.label("Items in inventory:");
                        for _ in items {
                            ui.label("some item");
                        }
                    });
            }
            UIState::No => {}
            UIState::Debug => {
                egui::Window::new("Debug")
                    .fixed_pos(Pos2::new(1., 1.))
                    .show(egui_ctx, |_ui| {});
            }
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context, _egui_ctx: &egui::CtxRef) -> anyhow::Result<()> {
        let (w, h) = window::get_size(ctx);
        self.scaler.set_outer_size(w, h);
        if self.is_needed_redraw && self.is_paused {
            graphics::set_canvas(ctx, self.scaler.canvas());
            graphics::clear(ctx, Color::rgb(0., 0., 0.));
            run_render_system(self, ctx).unwrap();
            graphics::reset_canvas(ctx);
            graphics::clear(ctx, Color::rgb(0., 0., 0.));
            self.is_needed_redraw = false;
        }
        self.scaler.draw(ctx);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context, _egui_ctx: &egui::CtxRef) -> anyhow::Result<()> {
        if self.is_paused {
            match self.next_action {
                PlayerAction::Move(dest) => {
                    let mut bind = self.world.query::<(&Player,)>();
                    let (e, _) = bind
                        .into_iter()
                        .next()
                        .expect("Персонаж потерялся. Как так?");
                    drop(bind);
                    self.world.insert(e, (WantsMove(dest),))?;
                    self.is_paused = false;
                    self.is_needed_redraw = true;
                }
                PlayerAction::OpenInventory => {
                    self.ui_state = UIState::Inventory {
                        items: get_player_items(&self.world)?,
                    }
                }
                PlayerAction::CloseInventory => {
                    self.ui_state = UIState::No;
                }
                PlayerAction::Nothing => {}
            }
            for system in self.game_systems.clone().iter() {
                system.run(self, ctx)?
            }
        } else {
            for system in self.world_systems.iter() {
                system.run(&mut self.world, ctx)?
            }
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
        ];
        let resources = Resources::load(ctx, &assets_path);
        let mut world = World::new();
        let map = WorldMap::new();
        world.spawn((map,));
        world.spawn(new_player());
        world.spawn((
            Item, //TODO: убрать эту отсебятину и сделать норм генератор предметов
            Name("item".into()),
            Renderable("item"),
            Position(Vec2::new(2, 2)),
        ));
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
    match ContextBuilder::new("S", 500, 500)
        .quit_on_escape(true)
        .resizable(true)
        .show_mouse(true)
        .grab_mouse(false)
        .key_repeat(true)
        //      .timestep(tetra::time::Timestep::Fixed(20.))
        .build()?
        .run(|ctx| Ok(StateWrapper::new(Game::new(ctx)?)))
    {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e); //FIXME: Вот так вот, надо выводить сообщения об ошибках до того как ебучий SDL крашнется из-за каких-то косяков в их деструкторе.
        }
    }
    Ok(())
}
