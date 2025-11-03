mod api;
mod body;
mod components;
mod inventory;
mod items;
mod map;
mod mob;
mod player;
mod resources;
mod systems;
mod tests;
mod ui;
use api::LuaApi;
use body::{Body, WantsAttack, Wound};
use components::Position;

use enum_unwrap::UnwrapEnum;
use hecs::{CommandBuffer, With, World};
use inventory::Inventory;
use items::Item;
use macroquad::{
    prelude::Color,
    window::{clear_background, next_frame, Conf},
};
use map::WorldMap;
use mob::{Log, Mob};
use player::{get_player_inventory, Player};
use resources::Resources;
use serde::Deserialize;
use std::{collections::HashMap, env, sync::Mutex, time::Duration};
use systems::{
    action::{Action, ActionKind, WorldTime},
    movement::dir_to_vec3,
    render::{run_render_system, Renderable},
    GameSystem, WorldSystem,
};

use ui::{set_skin, MenuState, UIConfig, UIState};

fn component_fmt<T: hecs::Component + std::fmt::Debug>(
    entity: hecs::EntityRef<'_>,
) -> Option<String> {
    Some(format!("{:#?}", entity.get::<&T>()?))
}

macro_rules! generate_debug_for_components {
    [$($x: ty),+ $(,)?] => {&[$(&component_fmt::<$x>),+] }
}

const FMT_COMPONENTS: &[FormattingFunction] = generate_debug_for_components![
    Log,
    // MapMemory,
    // Sight,
    Body,
    Renderable,
    Inventory,
    WantsAttack,
    Position,
    Player,
    Mob,
    Action
];

type FormattingFunction = &'static dyn Fn(hecs::EntityRef<'_>) -> Option<String>;

/// Функция для строкового представления всех компонентов в Entity
#[allow(dead_code)]
fn format_entity(entity: hecs::EntityRef<'_>) -> String {
    let mut out = String::new();
    for f in FMT_COMPONENTS {
        if let Some(x) = f(entity) {
            if out.is_empty() {
                out.push('[');
            } else {
                out.push_str(", ");
            }
            out.push_str(&x);
        }
    }
    if out.is_empty() {
        out.push_str("[]");
    } else {
        out.push(']');
    }
    out
}

type GameSystems = Vec<GameSystem>;
type WorldSystems = Vec<WorldSystem>;

type GameHasher = fxhash::FxBuildHasher;

pub fn hasher() -> GameHasher {
    GameHasher::default()
}

#[derive(Clone, Debug, PartialEq, Deserialize, UnwrapEnum)]
#[serde(untagged)]
pub enum Property {
    Int(i64),
    String(String),
    Float(f64),
    Map(HashMap<String, Property, GameHasher>),
    List(Vec<Property>),
}

//Переименовать
pub fn search<'a>(
    map: &'a HashMap<String, Property, GameHasher>,
    path: &[&str],
) -> Result<&'a Property, String> {
    let mut current = map;

    for &segment in path.iter().take(path.len().saturating_sub(1)) {
        match current.get(segment) {
            Some(Property::Map(submap)) => current = submap,
            Some(_) => return Err(format!("property {segment} is not a map")),
            _ => return Err(format!("property {segment} not found")),
        }
    }

    path.last()
        .ok_or("search path is empty".into())
        .and_then(|last_segment| {
            current
                .get(*last_segment)
                .ok_or(format!("property \"{last_segment}\" not found"))
        })
}

pub struct Game {
    api: LuaApi,
    world: World,
    resources: Resources,
    game_systems: GameSystems,
    world_systems: WorldSystems,
    ui: Option<UIState>,
    ui_config: UIConfig,
    next_action: PlayerAction,
    is_paused: bool,
    is_needed_redraw: bool,
    scale: f32,
    statistics: Mutex<Statistics>,
    show_stats: bool,
}

#[derive(Clone)]
pub struct Statistics {
    systems_average: HashMap<String, (Duration, u32), GameHasher>,
}

impl Default for Statistics {
    fn default() -> Self {
        Self::new()
    }
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            systems_average: HashMap::with_hasher(hasher()),
        }
    }
    pub fn show(&self) -> String {
        let mut total = Duration::default();
        let mut result = String::new();
        for (name, (stat, _)) in &self.systems_average {
            total += *stat;
            result.push_str(format!("{name} system elapsed: {stat:2?}\n").as_str());
        }
        result.push_str(format!("total systems elapsed: {total:2?}\n").as_str());
        result
    }
    pub fn update_stat(&mut self, time: Duration, system: String) {
        self.systems_average
            .entry(system)
            .and_modify(|(avg, counter)| {
                *counter += 1;
                *avg = ((*counter - 1) * *avg + time) / *counter;
                if *counter > 100 {
                    *counter = 1;
                    *avg = time;
                }
            })
            .or_insert((time, 1));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayerWorldAction {
    Move(Direction),
    OpenInventory,
    OpenLog,
    PickUpItem,
    Zoom,
    Unzoom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UIAction {
    Move(Direction),
    InventoryAction(InventoryAction),
    LogAction(LogAction),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InventoryAction {
    Close,
    DropItem,
    Equip,
    Dequip,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogAction {
    Close,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PlayerAction {
    Player(PlayerWorldAction),
    UI(UIAction),
    Nothing,
}

impl Game {
    async fn draw(&mut self) -> anyhow::Result<()> {
        clear_background(Color::from_hex(0x000000));
        let now = std::time::Instant::now();
        run_render_system(self)?;
        let elapsed = now.elapsed();
        let mut stats = self.statistics.lock().unwrap();
        stats.update_stat(elapsed, "Render system".into());

        Ok(())
    }

    async fn draw_ui(&self) -> anyhow::Result<()> {
        if let Some(ref ui) = self.ui {
            match ui {
                UIState::Inventory { ref state } => ui::hands_menu(state),
                UIState::Log { ref text } => ui::log(text),
            }
            if self.show_stats {
                ui::debug(&self.statistics.lock().unwrap());
            }
        }
        Ok(())
    }

    async fn update(&mut self) -> anyhow::Result<()> {
        if self.is_paused {
            self.handle_paused_state().await?;
        } else {
            self.run_world().await?;
        }
        Ok(())
    }

    async fn handle_paused_state(&mut self) -> anyhow::Result<()> {
        match self.next_action {
            PlayerAction::Player(action) => self.handle_player_world_action(action).await?,
            PlayerAction::UI(ref action) => {
                if let Some(ui) = &mut self.ui {
                    match ui {
                        UIState::Inventory { state: menu_state } => {
                            let mut bind = self.world.query::<With<&Inventory, &Player>>();
                            let (player_e, pos) =
                                bind.iter().last().expect("Персонаж потерялся. Как так?"); //TODO: привести сообщения об ошибках в порядок
                            let mut cmd = CommandBuffer::new();
                            match action {
                                UIAction::Move(Direction::Back) => menu_state.next_item(),
                                UIAction::Move(Direction::Forward) => menu_state.prev_item(),
                                UIAction::InventoryAction(InventoryAction::Equip) => {
                                    menu_state.holded_item = Some(menu_state.pointer);
                                    // inventory.holded = menu_state.holded_item;
                                    let action = Action::new(
                                        &self.world,
                                        1,
                                        ActionKind::Equip(menu_state.pointer),
                                    );
                                    cmd.insert(player_e, (action,));
                                    self.ui = None;
                                    self.is_paused = false;
                                    self.is_needed_redraw = true;
                                }
                                UIAction::InventoryAction(InventoryAction::Dequip) => {
                                    menu_state.holded_item = None;
                                    let action = Action::new(&self.world, 1, ActionKind::Dequip);
                                    cmd.insert(player_e, (action,));
                                    self.is_paused = false;
                                    self.is_needed_redraw = true;
                                }
                                UIAction::InventoryAction(InventoryAction::DropItem) => {
                                    if menu_state
                                        .holded_item
                                        .is_some_and(|x| x == menu_state.pointer)
                                    {
                                        menu_state.holded_item = None;
                                    }
                                    menu_state.items.remove(menu_state.pointer);
                                    let action = Action::new(
                                        &self.world,
                                        1,
                                        ActionKind::DropItem(menu_state.pointer),
                                    );
                                    cmd.insert(player_e, (action,));
                                    menu_state.pointer = menu_state.pointer.saturating_sub(1);
                                    self.is_paused = false;
                                    self.is_needed_redraw = true;
                                }
                                UIAction::InventoryAction(InventoryAction::Close) => self.ui = None,
                                _ => {}
                            }
                            drop(bind);
                            cmd.run_on(&mut self.world);
                        }
                        UIState::Log { .. } => {
                            if let UIAction::LogAction(LogAction::Close) = action {
                                self.ui = None
                            }
                        }
                    }
                }
            }
            PlayerAction::Nothing => {}
        }
        for system in self.game_systems.clone().iter() {
            system.run(self)?
        }
        Ok(())
    }

    async fn handle_player_world_action(
        &mut self,
        action: PlayerWorldAction,
    ) -> anyhow::Result<()> {
        match action {
            PlayerWorldAction::Move(dir) => {
                let mut bind = self.world.query::<With<&Position, &Player>>();
                let (player_e, Position(pos)) = bind
                    .into_iter()
                    .next()
                    .expect("Персонаж потерялся. Как так?");
                let pos = *pos;
                drop(bind);
                let mut mobs = self.world.query::<With<&Position, &Mob>>();
                let mut cmd = CommandBuffer::new();
                if let Some((target, _)) = mobs
                    .iter()
                    .find(|(_, Position(mob_pos))| *mob_pos == pos + dir_to_vec3(&dir))
                {
                    let action =
                        Action::new(&self.world, 3, ActionKind::Attack(Wound::Bruised, target));
                    cmd.insert(player_e, (action,));
                } else {
                    let action = Action::new(&self.world, 3, ActionKind::Move(dir));
                    cmd.insert(player_e, (action,));
                }
                drop(mobs);
                cmd.run_on(&mut self.world);
                self.is_paused = false;
                self.is_needed_redraw = true;
            }
            PlayerWorldAction::OpenInventory => {
                let inventory = get_player_inventory(&self.world)?;
                dbg!(&inventory);
                let state = MenuState::new(inventory.items, inventory.holded);
                self.ui = Some(UIState::Inventory { state });
            }
            PlayerWorldAction::PickUpItem => {
                let mut bind_player = self.world.query::<With<&Position, &Player>>();
                let (player_e, player_pos) = bind_player
                    .into_iter()
                    .next()
                    .expect("Персонаж потерялся. Как так?");

                let mut bind_item = self.world.query::<With<&Position, &Item>>();
                let item = bind_item
                    .into_iter()
                    .filter(|(_, pos)| *pos == player_pos)
                    .next();
                if let Some(item) = item {
                    let item_e = item.0;
                    let _ = (bind_item, bind_player);
                    let action = Action::new(&self.world, 1, ActionKind::PickUp(item_e));
                    self.world.insert(player_e, (action,)).unwrap();
                }

                self.is_paused = false;
                self.is_needed_redraw = true;
            }
            PlayerWorldAction::OpenLog => {
                let mut bind_player = self.world.query::<With<&Log, &Player>>();
                let (_, log) = bind_player
                    .into_iter()
                    .next()
                    .expect("Персонаж потерялся. Как так?");
                self.ui = Some(UIState::Log {
                    text: log.0.clone(),
                })
            }
            PlayerWorldAction::Zoom => {
                self.scale += 0.1;
            }
            PlayerWorldAction::Unzoom => {
                self.scale -= 0.1;
            }
        }
        Ok(())
    }

    async fn run_world(&mut self) -> anyhow::Result<()> {
        let player_e = self.world.query::<&Player>().iter().next().unwrap().0;
        let player_e = self.world.entity(player_e).unwrap();
        println!(
            "{}",
            component_fmt::<Action>(player_e).unwrap_or("no action".into())
        );
        for system in self.world_systems.iter() {
            let now = std::time::Instant::now();
            system.run(&mut self.world)?;
            let elapsed = now.elapsed();
            let mut stats = self.statistics.lock().unwrap();
            stats.update_stat(elapsed, format!("{system:?}"));
        }
        self.is_paused = self.is_player_move();
        Ok(())
    }

    pub fn is_player_move(&self) -> bool {
        self.world
            .query::<&Player>()
            .with::<&Action>()
            .iter()
            .last()
            .is_none()
    }

    async fn new() -> anyhow::Result<Game> {
        set_skin().await;
        let exe_path = env::current_exe().expect("Ты ебанутый? Ты что там делаешь?");
        let data_path = exe_path
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("data"))
            .filter(|p| p.exists())
            .unwrap_or(env::current_dir().expect("Ты как сюда залез?").join("data"));
        let game_systems: GameSystems = vec![GameSystem::InputSystem];
        let world_systems: WorldSystems = vec![
            // WorldSystem::Move,
            WorldSystem::Action,
            WorldSystem::FovCompute,
            WorldSystem::Memory,
            WorldSystem::Pathfinding,
            // WorldSystem::Attack,
        ];
        let mut world = World::new();
        let mut resources = Resources::new();
        let mod_api = LuaApi::init(&mut world, &mut resources).await;
        world.spawn((WorldTime(0),));
        let map = WorldMap::new();
        world.spawn((map,));

        Ok(Game {
            api: mod_api,
            world,
            resources,
            game_systems,
            world_systems,
            ui: None,
            ui_config: UIConfig::default(),
            next_action: PlayerAction::Nothing,
            is_paused: false,
            is_needed_redraw: true,
            scale: 1.,
            statistics: Mutex::new(Statistics::new()),
            show_stats: false,
        })
    }
}

fn window_conf() -> Conf {
    use macroquad::miniquad;
    Conf {
        window_title: "Window Conf".to_owned(),
        fullscreen: false,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::X11Only,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> anyhow::Result<()> {
    let mut game = Game::new().await?;
    loop {
        let now = std::time::Instant::now();
        game.update().await?;
        game.draw().await?;
        game.draw_ui().await?;
        next_frame().await;
        let _elapsed = now.elapsed();
    }
}
